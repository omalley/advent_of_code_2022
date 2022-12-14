use std::cmp::min;
use std::time::Duration;
use std::time::Instant;

use argh::FromArgs;
use colored::*;

use omalley_aoc2022::*;

#[derive(FromArgs)]
/** Advent of Code (https://adventofcode.com/)
*/
struct Args {
    /// A single day to execute (all days by default)
    #[argh(option, short = 'd')]
    day: Option<usize>,
}

fn pretty_print(line: &str, output: Option<&str>, duration: Duration) {
    const DISPLAY_WIDTH: usize = 40;

    let duration = format!("({:.2?})", duration);
    print!("{} {}", line, duration.dimmed());

    match output {
        Some(output) => {
            let width = "  - ".len() + line.chars().count() + 1 + duration.chars().count();
            let dots = DISPLAY_WIDTH - min(DISPLAY_WIDTH - 5, width) - 2;
            print!(" {}", ".".repeat(dots).dimmed());

            if output.contains('\n') {
                println!();

                for line in output.trim_matches('\n').lines() {
                    println!("    {}", line.bold());
                }
            } else {
                println!(" {}", output.bold());
            }
        }
        None => println!(),
    }
}

// Time the given function, returning its result and the elapsed time
fn time<T>(func: &dyn Fn() -> T) -> (Duration, T) {
    let start = Instant::now();
    let result = func();

    (start.elapsed(), result)
}

/// Stolen from https://github.com/remi-dupre/aoc and heavily modified for simplicity.
#[macro_export]
macro_rules! main {
    (
        year: $year: expr,
        implemented_days: [$($day:ident),+ $(,)?]
    ) => {
        // Inputs need to be in this format to work with `cargo aoc input`.
        const DAYS: &[&str] = &[$(stringify!($day)),*];
        const INPUTS : &[&str] = &[$(include_str!(concat!("../input/", stringify!($year), "/", stringify!($day), ".txt"))),*];

        fn main() {
            let args: Args = argh::from_env();

            let (elapsed, _) = time(&|| {
                let days = match args.day {
                    Some(day) => {
                        assert!(DAYS.contains(&format!("day{}", day).as_ref()), "Requested an unimplemented day");
                        vec![day]
                    },
                    None => DAYS.iter().map(|s| s[3..].parse().expect("Weird looking day")).collect()
                };

                for day in days.into_iter() {
                    let module_name = format!("day{}", day);

                    match module_name.as_ref() {
                        $(stringify!($day) => {
                            let data = INPUTS[day as usize - 1];

                            let (gen_elapsed, input) = time(&|| $day::generator(&data));
                            let (p1_elapsed, p1_result) = time(&|| $day::part1(&input));
                            let (p2_elapsed, p2_result) = time(&|| $day::part2(&input));

                            let duration = format!("({:.2?})", gen_elapsed + p1_elapsed + p2_elapsed);
                            println!("{} {}", format!("Day {}", day).bold(), duration.dimmed());
                            pretty_print(" ?? Generator", None, gen_elapsed);
                            pretty_print(" ?? Part 1", Some(&format!("{}", p1_result)), p1_elapsed);
                            pretty_print(" ?? Part 2", Some(&format!("{}", p2_result)), p2_elapsed);

                            // Break up whatever comes after us
                            println!()
                        },)+
                        _ => unreachable!() // All the days should've been hit by the match
                    }
                }
            });

            println!("{} {}", "Overall runtime".bold(), format!("({:.2?})", elapsed).dimmed());
        }
    };
}

main! {
    year: 2022,
    implemented_days: [
        day1,
        day2,
        day3,
        day4,
        day5,
        day6,
        day7,
        day8,
        day9,
        day10,
        day11,
        day12,
        day13,
        day14,
        day15,
        day16,
        day17,
        day18,
        day19,
        day20,
        day21,
        day22,
        day23,
        day24,
        day25,
    ]
}
