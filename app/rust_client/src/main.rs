use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::{pubkey, signer::keypair};
use anchor_client::{Client, Cluster};
use std::process::exit;
use std::rc::Rc;
use std::str::FromStr;

use switchboard_feed_solana::accounts as ReadResult_Acc;
use switchboard_feed_solana::instruction as ReadResult_Ins;
use switchboard_feed_solana::SolanaPriceFeed;
// GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR

//  feed_vector :9tTV1NGmLXH2vtMt56MZJYukyBLJzN6sAYwHTVeGfGvD
fn main() {
    let mut feed_vector_pubkey: Vec<pubkey::Pubkey> = Vec::new();
    feed_vector_pubkey.push(pubkey::Pubkey::from_str(FEED_VEC).unwrap());
    loop {
        println!("Running..");
        println!(
            "1:Create Price feed Vector Account to store Price Feeds\n\
        2:Read From History Buffer and append to Price Feed Vector\n\
        3:Calculate Data Spread of Price Feed (Standard Deviation per second of Solana price)\n\
        4:Reset the price feed Vector(empty)\n\
        5:Exit"
        );
        println!("Enter your option:");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).expect("err");
        let option: u8 = buf.trim().parse().unwrap();
        match option {
            1u8 => instructions::create_feed_vec(),
            2u8 => instructions::read_sol_feed(),
            3u8 => instructions::cal_data_spread(),
            4u8 => instructions::reset_feed_vec_acc(),
            5u8 => {
                exit(0);
            }
            _ => {
                println!("Invalid Option!")
            }
        }
    }
}

const HISTORY_BUFFER: &str = "7LLvRhMs73FqcLkA8jvEE1AM2mYZXTmqfUv8GAEurymx";
const SOLANA_FEED_PID: &str = "GWLjdS5qvUUwTt8HHVTHmJ8F4ZmNYRvdNoiL8gBgc5H7";
const KEYPAIR_PATH: &str = "ENTER SOLANA ID PATH HERE";
const FEED_VEC: &str = "9xHLTg2uoRUhADhb6c6VJfoPi7kohorhSQoGn5NFS4xo";

mod instructions {
    use super::*;
    pub fn create_feed_vec() {
        let feed_pid = pubkey::Pubkey::from_str(SOLANA_FEED_PID).unwrap();

        let payer_keypair =
            keypair::read_keypair_file(KEYPAIR_PATH).expect("Error in reading Keypair Path");

        let payer_pubkey = payer_keypair.try_pubkey().unwrap();

        let rpc = Client::new_with_options(
            Cluster::Devnet,
            Rc::new(payer_keypair),
            CommitmentConfig::processed(),
        );
        let feed_vector_keypair = anchor_client::solana_sdk::signer::keypair::Keypair::new();
        let feed_vector_pubkey = feed_vector_keypair.pubkey();

        let program = rpc.program(feed_pid);
        let sig = program
            .request()
            .accounts(ReadResult_Acc::CreatePrizeFeedAccount {
                system_program: anchor_client::solana_sdk::system_program::id(),
                feed_vector_acc: feed_vector_pubkey,
                authority: payer_pubkey,
            })
            .args(ReadResult_Ins::CreatePriceFeed {})
            .signer(&feed_vector_keypair)
            .send()
            .unwrap();
        println!("FeedVectorAcc Created! with Siganture:{:?}", sig);
        println!("FeedVectorPubkey:{:?}", feed_vector_pubkey);
    }
    pub fn read_sol_feed() {
        let mut period = String::new();
        println!("Enter the period in seconds:");
        std::io::stdin().read_line(&mut period).expect("err");
        let period = period.trim().parse::<i64>().unwrap();

        let histor_buffer_pubkey = pubkey::Pubkey::from_str(HISTORY_BUFFER).unwrap();

        let feed_pid = pubkey::Pubkey::from_str(SOLANA_FEED_PID).unwrap();
        let feed_vec_pubkey = pubkey::Pubkey::from_str(FEED_VEC).unwrap();

        let payer_keypair =
            keypair::read_keypair_file(KEYPAIR_PATH).expect("Error in reading Keypair Path");

        let payer_pubkey = payer_keypair.try_pubkey().unwrap();

        let rpc = Client::new_with_options(
            Cluster::Devnet,
            Rc::new(payer_keypair),
            CommitmentConfig::processed(),
        );

        let program = rpc.program(feed_pid);
        let sig1 = program
            .request()
            .accounts(ReadResult_Acc::ReadHistorybuffer {
                history_buffer: histor_buffer_pubkey,
                feed_vec_acc: feed_vec_pubkey,
                authority: payer_pubkey,
            })
            .args(ReadResult_Ins::AppendFeedData { period: period })
            .send()
            .unwrap();

        let vec_feed_acc: SolanaPriceFeed = program.account(feed_vec_pubkey).unwrap();

        println!("Signature:{:?}", sig1);
        println!("FeedVec:");
        for i in vec_feed_acc.feed_vector.iter() {
            println!("{:?}", i);
        }
        println!("authority:{:?}", vec_feed_acc.authority);
    }

    pub fn cal_data_spread() {
        let mut days = String::new();
        println!("ENter the number of days:");
        std::io::stdin().read_line(&mut days).expect("err");
        let days = days.trim().parse::<u64>().unwrap();
        let feed_pid = pubkey::Pubkey::from_str(SOLANA_FEED_PID).unwrap();
        let feed_vec_pubkey = pubkey::Pubkey::from_str(FEED_VEC).unwrap();
        let payer_keypair =
            keypair::read_keypair_file(KEYPAIR_PATH).expect("Error in reading Keypair Path");

        let payer_pubkey = payer_keypair.try_pubkey().unwrap();

        let rpc = Client::new_with_options(
            Cluster::Devnet,
            Rc::new(payer_keypair),
            CommitmentConfig::processed(),
        );

        let program = rpc.program(feed_pid);
        let sig1 = program
            .request()
            .accounts(ReadResult_Acc::CalculateDataSpread {
                feed_vector_acc: feed_vec_pubkey,
                authority: payer_pubkey,
            })
            .args(ReadResult_Ins::CalculateDataSpread { nod: days })
            .send()
            .unwrap();
        let feed_vec_acc: SolanaPriceFeed =
            program.account(feed_vec_pubkey.to_owned()).expect("err");

        println!("Signature of data spread:{:?}", sig1);
        println!("Price feed Vector:");
        for i in feed_vec_acc.feed_vector.iter() {
            println!("{:?}", i);
        }
        println!("Data Spread:{:?}", feed_vec_acc.data_spread);
    }

    pub fn reset_feed_vec_acc() {
        let feed_pid = pubkey::Pubkey::from_str(SOLANA_FEED_PID).unwrap();
        let feed_vec_pubkey = pubkey::Pubkey::from_str(FEED_VEC).unwrap();
        let payer_keypair =
            keypair::read_keypair_file(KEYPAIR_PATH).expect("Error in reading Keypair Path");

        let payer_pubkey = payer_keypair.try_pubkey().unwrap();

        let rpc = Client::new_with_options(
            Cluster::Devnet,
            Rc::new(payer_keypair),
            CommitmentConfig::processed(),
        );

        let program = rpc.program(feed_pid);
        let sig = program
            .request()
            .accounts(ReadResult_Acc::ResetFeedVec {
                feed_vec_acc: feed_vec_pubkey,
                authority: payer_pubkey,
            })
            .args(ReadResult_Ins::ResetVecFeed {})
            .send()
            .unwrap();
        let feed_vec_acc: SolanaPriceFeed =
            program.account(feed_vec_pubkey.to_owned()).expect("err");
        println!("Feed Vector has been reset with Signature:{:?}", sig);
        println!("FeedVector:{:?}", feed_vec_acc.feed_vector);
    }
}
