use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rayon::prelude::*;

#[derive(Debug, PartialEq)]
struct Game<'a> {
    name: &'a str,
    price: f32,
}

const GAMES_COUNT: usize = 43;
const ALPHA: f32 = 0.75;
const BUDGET: f32 = 150.0;
const NBR_JEUX_MAX: usize = 12;

fn get_liking(selection: &[usize], notes: &[Vec<Option<f32>>], means: &[f32]) -> f32 {
    let mut likings: Vec<(usize, f32)> = vec![(0, 0.0); notes.len()];
    for i in selection {
        for (user_index, user) in notes.iter().enumerate() {
            if let Some(note) = user[*i] {
                if note > means[user_index] {
                    likings[user_index].1 +=
                        (note - means[user_index]) * ALPHA.powf(likings[user_index].0 as f32);
                    likings[user_index].0 += 1;
                }
            }
        }
    }
    likings.iter().fold(0.0, |acc, (_, liking)| acc + liking)
}

fn budget(games: &[Game<'_>], selection: &[usize]) -> f32 {
    selection
        .iter()
        .fold(0.0, |x, &index| x + games[index].price)
}

fn count_combinations(n: u64, r: u64) -> u64 {
    if r > n {
        0
    } else {
        (1..=r.min(n - r)).fold(1, |acc, val| acc * (n - val + 1) / val)
    }
}

fn main() {
    let source = include_str!("../data.csv");

    let games: Vec<Option<Game>> = include_str!("../jeux.csv")
        .lines()
        .map(|l| {
            let (title, price) = l
                .split(',')
                .collect_tuple::<(&str, &str)>()
                .expect("wrong line");
            Some(Game {
                name: title,
                price: price.parse::<f32>().ok()?,
            })
        })
        .collect();

    let notes: Vec<Vec<Option<f32>>> = source
        .lines()
        .map(|l| {
            l.split(',')
                .skip(2)
                .take(GAMES_COUNT)
                .map(|n| n.parse::<f32>().ok())
                .zip(games.iter())
                .filter_map(|(note, game)| if game.is_some() { Some(note) } else { None })
                .collect::<Vec<Option<f32>>>()
        })
        .collect();

    let games: Vec<Game> = games.into_iter().flatten().collect();

    println!(
        "{} games have a price and were selected as candidates",
        games.len()
    );

    let means: Vec<f32> = notes
        .iter()
        .map(|l| {
            let (sum, count) = l.iter().fold((0.0, 0.0), |(acc_val, acc_sum), c| {
                if let Some(v) = c {
                    (acc_val + v, acc_sum + 1.0)
                } else {
                    (acc_val, acc_sum)
                }
            });
            sum / count
        })
        .collect();

    let mut best_liking = 0.0;
    let mut best_price = 0.0;
    let mut best_combination: Option<Vec<usize>> = None;
    let indices: Vec<usize> = (0..games.len()).collect();

    for nbr in 0..NBR_JEUX_MAX {
        println!("Testing {nbr} games");
        let combinations: Vec<Vec<&usize>> = indices.iter().combinations(nbr).collect();

        let combinations = combinations
            // .into_par_iter()
            .into_iter()
            .progress()
            // .progress_count(count_combinations(games.len() as u64, nbr as u64))
            .filter(|combination| {
                let price = budget(
                    &games,
                    &combination.iter().map(|&x| *x).collect::<Vec<usize>>(),
                );
                price < BUDGET
            })
            .map(|x| x.iter().map(|&y| *y).collect::<Vec<usize>>());

        for combination in combinations {
            for perm in combination
                .iter()
                .permutations(combination.len())
                .map(|x| x.iter().map(|&y| *y).collect::<Vec<usize>>())
            {
                let liking = get_liking(&combination, &notes, &means);
                if liking > best_liking {
                    let price = budget(&games, &perm);
                    best_liking = liking;
                    best_price = price;
                    best_combination = Some(perm);
                }
            }
        }

        println!("Best liking is {best_liking}");
        println!("Best price is {best_price}");
        if let Some(comb) = &best_combination {
            let best_games: Vec<&str> = comb.iter().map(|&i| games[i].name).collect();
            println!("Best combination is {best_games:?}");
        }
        println!();
    }

    println!("Best liking is {best_liking}");
    println!("Best price is {best_price}");
    if let Some(comb) = best_combination {
        let best_games: Vec<&str> = comb.iter().map(|&i| games[i].name).collect();
        println!("Best combination is {best_games:?}");
    }
}
