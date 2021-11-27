use {
    clap::{
        crate_description, crate_name, crate_version, value_t_or_exit, App, AppSettings, Arg,
        ArgMatches, SubCommand,
    },
    solana_clap_utils::{
        input_parsers::{keypair_of, pubkey_of},
        input_validators::{is_keypair, is_url, is_valid_pubkey, is_within_range},
        keypair::{signer_from_path, CliSignerInfo},
    },
    solana_client::rpc_client::RpcClient,
    solana_remote_wallet::remote_wallet::RemoteWalletManager,
    solana_scoring::{
        id,
        state::{Mint, MintState},
        utils::try_from_slice_checked,
    },
    solana_sdk::{
        commitment_config::CommitmentConfig,
        native_token::lamports_to_sol,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    },
    std::fmt::Display,
    std::{collections::HashMap, process::exit, str::FromStr, sync::Arc},
};

struct Config {
    keypair: Keypair,
    json_rpc_url: String,
    verbose: bool,
}

pub fn is_short<T>(string: T) -> Result<(), String>
where
    T: AsRef<str> + Display,
{
    if string.as_ref().len() >= 256 {
        return Err(format!("too long: {}", string));
    }
    Ok(())
}

pub fn is_short_url<T>(string: T) -> Result<(), String>
where
    T: AsRef<str> + Display,
{
    // inlining is_url
    match url::Url::parse(string.as_ref()) {
        Ok(url) => {
            if url.has_host() {
            } else {
                return Err("no host provided".to_string());
            }
        }
        Err(err) => {
            return Err(format!("{}", err));
        }
    }
    is_short(string)?;
    Ok(())
}

fn new_throwaway_signer() -> (Box<dyn Signer>, Pubkey) {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    (Box::new(keypair) as Box<dyn Signer>, pubkey)
}

fn get_signer(
    matches: &ArgMatches<'_>,
    keypair_name: &str,
    wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
) -> Option<(Box<dyn Signer>, Pubkey)> {
    matches.value_of(keypair_name).map(|path| {
        let signer =
            signer_from_path(matches, path, keypair_name, wallet_manager).unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                exit(1);
            });
        let signer_pubkey = signer.pubkey();
        (signer, signer_pubkey)
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wallet_manager = None;
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("keypair")
                .long("keypair")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .global(true)
                .help("Filepath or URL to a keypair [default: client keypair]"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster [default: value from configuration file]"),
        )
        .subcommand(
            SubCommand::with_name("get-mint-details")
                .about("Display information about the given scoring mint")
                .arg(
                    Arg::with_name("mint_address")
                        .value_name("MINT_ADDRESS")
                        .validator(is_valid_pubkey)
                        .index(1)
                        .required(true)
                        .help("The address of the mint to be shown"),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-scoring-mint")
                .about("Create a mint for a new score type")
                .arg(
                    Arg::with_name("mint_keypair")
                        .value_name("MINT_KEYPAIR")
                        .validator(is_keypair)
                        .index(1)
                        .help(
                            "The keypair for the mint to be created. \
                              [default: randomly generated keypair]",
                        ),
                )
                .arg(
                    Arg::with_name("scoring_authority")
                        .long("scoring-authority")
                        .alias("keypair")
                        .value_name("ADDRESS")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help(
                            "Specify the scoring authority address. \
                             Defaults to the client keypair address.",
                        ),
                )
                .arg(
                    Arg::with_name("freeze_authority")
                        .long("freeze-authority")
                        .value_name("ADDRESS")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help("Specify the freeze authority address. Defaults to unset."),
                )
                .arg(
                    Arg::with_name("metadata_uri")
                        .long("uri")
                        .value_name("ADDRESS")
                        .validator(is_short_url)
                        .takes_value(true)
                        .help(
                            "Specify the JSON URI containing metadata for the score. \
                             URI may be no longer than 255 bytes.",
                        ),
                ),
        )
        .get_matches();

    let (sub_command, sub_matches) = app_matches.subcommand();
    let matches = sub_matches.unwrap();

    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        Config {
            json_rpc_url: matches
                .value_of("json_rpc_url")
                .unwrap_or(&cli_config.json_rpc_url)
                .to_string(),
            keypair: read_keypair_file(
                matches
                    .value_of("keypair")
                    .unwrap_or(&cli_config.keypair_path),
            )?,
            verbose: matches.is_present("verbose"),
        }
    };
    solana_logger::setup_with_default("solana=info");
    let rpc_client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), CommitmentConfig::confirmed());

    match (sub_command, sub_matches) {
        ("get-mint-details", Some(arg_matches)) => {
            // let user_address =
            //     pubkey_of(arg_matches, "user_address").unwrap_or(config.keypair.pubkey());
            // let house_addr = nobilitydao::get_house_address(&user_address);
            // println!("House Address: {}", house_addr);
            // let housedata = get_house(&rpc_client, &house_addr)?;
            // let coa_url = housedata.coat_of_arms;
            // let display_name = housedata.display_name;
            // println!("Display Name: {}", display_name);
            // println!("Coat of Arms: {}", coa_url);
            Ok(())
        }
        ("create-scoring-mint", Some(arg_matches)) => {
            let user_keypair = config.keypair;
            let (mint_signer, mint) = get_signer(arg_matches, "token_keypair", &mut wallet_manager)
                .unwrap_or_else(new_throwaway_signer);
            let scoring_authority = pubkey_of(arg_matches, "scoring_authority").unwrap_or(user_keypair.pubkey());
            let freeze_pubkey;
            let mut freeze_authority: Option<&Pubkey> = None;
            if arg_matches.is_present("freeze_authority") {
                freeze_pubkey = pubkey_of(arg_matches, "freeze_authority").unwrap();
                freeze_authority = Some(&freeze_pubkey);
            }
            let metadata_uri = arg_matches.value_of("metadata_uri").unwrap();
            let minimum_balance_for_rent_exemption =
                rpc_client.get_minimum_balance_for_rent_exemption(Mint::SIZE)?;

            let mut transaction = Transaction::new_with_payer(
                &[
                    system_instruction::create_account(
                        &user_keypair.pubkey(),
                        &mint,
                        minimum_balance_for_rent_exemption,
                        Mint::SIZE as u64,
                        &id(),
                    ),
                    solana_scoring::instruction::initialize_score_mint(
                        &id(),
                        &mint,
                        &scoring_authority,
                        freeze_authority,
                        metadata_uri.to_string(),
                    )?,
                ],
                Some(&user_keypair.pubkey()),
            );
            let blockhash = rpc_client.get_recent_blockhash()?.0;
            transaction.try_sign(&[&user_keypair, mint_signer.as_ref()], blockhash)?;

            rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;
            println!("Done creating scoring mint");
            Ok(())
        }
        _ => unreachable!(),
    }
}

// fn get_house(rpc_client: &RpcClient, house_address: &Pubkey) -> Result<HouseData, String> {
//     let account = rpc_client
//         .get_multiple_accounts(&[*house_address])
//         .map_err(|err| err.to_string())?
//         .into_iter()
//         .next()
//         .unwrap();

//     match account {
//         None => Err(format!("House {} does not exist", house_address)),
//         Some(account) => try_from_slice_checked::<HouseData>(&account.data, HouseData::SIZE)
//             .map_err(|err| format!("Failed to deserialize house {}: {}", house_address, err)),
//     }
// }

// fn create_house(
//     rpc_client: &RpcClient,
//     user_keypair: &Keypair,
//     coat_of_arms_str: &str,
//     display_name_str: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let house_addr = nobilitydao::get_house_address(&user_keypair.pubkey());
//     println!("House Address: {}", house_addr);

//     let mut transaction = Transaction::new_with_payer(
//         &[nobilitydao::instruction::create_house(
//             &user_keypair.pubkey(),
//             &house_addr,
//             coat_of_arms_str.to_string(),
//             display_name_str.to_string(),
//         )],
//         Some(&user_keypair.pubkey()),
//     );
//     let blockhash = rpc_client.get_recent_blockhash()?.0;
//     transaction.try_sign(&[user_keypair], blockhash)?;

//     rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;
//     println!("Done creating house!");
//     Ok(())
// }
