// Import necessary crates
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use rand::seq::SliceRandom;
use rand::thread_rng;
use polars::prelude::*;

// Define the categories and their kana
const MAIN_KANA: &[&str] = &["あ", "い", "う", "え", "お", "か", "き", "く", "け", "こ", "さ", "し", "す", "せ", "そ"];
const DAKUTEN_KANA: &[&str] = &["が", "ぎ", "ぐ", "げ", "ご", "ざ", "じ", "ず", "ぜ", "ぞ"];
const COMBINATION_KANA: &[&str] = &["きゃ", "きゅ", "きょ", "しゃ", "しゅ", "しょ", "じゃ", "じゅ", "じょ"];

fn main() {
    let conversion_table = kana_to_romaji();

    println!("Welcome to the Kana Reaction Time Tester!");
    println!("Choose a category to test:");
    println!("1. Main Kana");
    println!("2. Dakuten Kana");
    println!("3. Combination Kana");

    let category = get_user_choice();
    let kana_set = match category {
        1 => MAIN_KANA,
        2 => DAKUTEN_KANA,
        3 => COMBINATION_KANA,
        _ => unreachable!(),
    };

    println!("You selected category {}. Press Enter to start.", category);
    let _ = io::stdin().read_line(&mut String::new());

    let (kana_results, reaction_times, attempts) = test_reaction_times(kana_set, &conversion_table);

    // Prepare data for Polars
    let reaction_time_column: Vec<f64> = reaction_times.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    let df = DataFrame::new(
        vec![
            Series::new("Kana".into(), &kana_results).into(),
            // Series::new("Reaction Time (ms)", reaction_time_column),
            Series::new("Reaction Time (ms)".into(), &reaction_time_column).into(),
            // Series::new("Attempts", attempts),
            Series::new("Attempts".into(), &attempts).into(),
        ]
    )
    .unwrap();

    println!("\nTest complete! Here are your results:\n");
    println!("{:?}", df);

    // Calculate average reaction time
    let total_time: Duration = reaction_times.iter().sum();
    let average_time = total_time / reaction_times.len() as u32;

    println!("\nAverage reaction time: {:.2} ms", average_time.as_secs_f64() * 1000.0);

    // Prepare data for Polars
    let kana_column: Vec<String> = kana_set.iter().cycle().take(reaction_times.len()).map(|&s| s.to_string()).collect();
    let reaction_time_column: Vec<f64> = reaction_times.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    let attempts_column: Vec<u32> = attempts.clone();

    let df = DataFrame::new(
        vec![
            Series::new("Kana".into(), &kana_column).into(),
            Series::new("Reaction Time (ms)".into(), &reaction_time_column).into(),
            Series::new("Attempts".into(), &attempts_column).into(),
        ]
    ).unwrap();

    println!("\nTest complete! Here are your results:\n");
    println!("{}", df);

    // Calculate average reaction time for correct attempts
    let total_time: Duration = reaction_times.iter().sum();
    let average_time = total_time / reaction_times.len() as u32;

    println!("\nAverage reaction time: {:.2} ms", average_time.as_secs_f64() * 1000.0);
}

fn kana_to_romaji() -> HashMap<String, String> {
    let table: HashMap<String, String> = [
        // Main Kana
        ("あ", "a"), ("い", "i"), ("う", "u"), ("え", "e"), ("お", "o"),
        ("か", "ka"), ("き", "ki"), ("く", "ku"), ("け", "ke"), ("こ", "ko"),
        ("さ", "sa"), ("し", "shi"), ("す", "su"), ("せ", "se"), ("そ", "so"),
        ("た", "ta"), ("ち", "chi"), ("つ", "tsu"), ("て", "te"), ("と", "to"),
        ("な", "na"), ("に", "ni"), ("ぬ", "nu"), ("ね", "ne"), ("の", "no"),
        ("は", "ha"), ("ひ", "hi"), ("ふ", "fu"), ("へ", "he"), ("ほ", "ho"),
        ("ま", "ma"), ("み", "mi"), ("む", "mu"), ("め", "me"), ("も", "mo"),
        ("や", "ya"), ("ゆ", "yu"), ("よ", "yo"), ("ら", "ra"), ("り", "ri"), ("る", "ru"), ("れ", "re"), ("ろ", "ro"), ("わ", "wa"), ("を", "wo"), ("ん", "n"),
        // Dakuten Kana
        ("が", "ga"), ("ぎ", "gi"), ("ぐ", "gu"), ("げ", "ge"), ("ご", "go"),
        ("ざ", "za"), ("じ", "ji"), ("ず", "zu"), ("ぜ", "ze"), ("ぞ", "zo"),
        ("だ", "da"), ("ぢ", "di"), ("づ", "du"), ("で", "de"), ("ど", "do"),
        ("ば", "ba"), ("び", "bi"), ("ぶ", "bu"), ("べ", "be"), ("ぼ", "bo"),
        // Handakuten Kana
        ("ぱ", "pa"), ("ぴ", "pi"), ("ぷ", "pu"), ("ぺ", "pe"), ("ぽ", "po"),
        // Combination Kana
        ("きゃ", "kya"), ("きゅ", "kyu"), ("きょ", "kyo"),
        ("しゃ", "sha"), ("しゅ", "shu"), ("しょ", "sho"),
        ("ちゃ", "cha"), ("ちゅ", "chu"), ("ちょ", "cho"),
        ("にゃ", "nya"), ("にゅ", "nyu"), ("にょ", "nyo"),
        ("ひゃ", "hya"), ("ひゅ", "hyu"), ("ひょ", "hyo"),
        ("みゃ", "mya"), ("みゅ", "myu"), ("みょ", "myo"),
        ("りゃ", "rya"), ("りゅ", "ryu"), ("りょ", "ryo"),
        ("ぎゃ", "gya"), ("ぎゅ", "gyu"), ("ぎょ", "gyo"),
        ("じゃ", "ja"), ("じゅ", "ju"), ("じょ", "jo"),
        ("びゃ", "bya"), ("びゅ", "byu"), ("びょ", "byo"),
        ("ぴゃ", "pya"), ("ぴゅ", "pyu"), ("ぴょ", "pyo")
    ].iter().cloned().map(|(k, v)| (k.to_string(), v.to_string())).collect();

    table
}

fn get_user_choice() -> u32 {
    loop {
        print!("Enter your choice (1/2/3): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<u32>() {
            Ok(choice) if (1..=3).contains(&choice) => return choice,
            _ => println!("Invalid choice. Please enter 1, 2, or 3."),
        }
    }
}
fn test_reaction_times(
    kana_set: &[&str],
    conversion_table: &HashMap<String, String>,
) -> (Vec<String>, Vec<Duration>, Vec<u32>) {
    let mut rng = thread_rng();
    let mut reaction_times = Vec::new();
    let mut attempts = Vec::new();
    let mut kana_results = Vec::new();

    for _ in 0..10 {
        let kana = kana_set.choose(&mut rng).unwrap();
        let expected = conversion_table.get(&kana.to_string()).unwrap();
        let mut attempt_count = 0;

        loop {
            println!("Kana: {}\nType the corresponding romaji!", kana);

            let start_time = Instant::now();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let elapsed = start_time.elapsed();

            attempt_count += 1;
            if input.trim() == expected {
                println!(
                    "\x1b[1mCorrect! Reaction time: {:.2} ms\x1b[0m",
                    elapsed.as_secs_f64() * 1000.0
                );
                reaction_times.push(elapsed);
                attempts.push(attempt_count);
                kana_results.push(kana.to_string());
                break;
            } else {
                println!("\x1b[1;31mIncorrect! Try again.\x1b[0m");
            }
        }
    }

    (kana_results, reaction_times, attempts)
}
