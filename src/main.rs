use chrono::prelude::*;
use itertools::Itertools;


struct TimeSignal {
    name: String,
    value: u32,
    trigger : chrono::DateTime<chrono::Utc>,
}

struct Day2Day<'a> {
    bars : (&'a yahoo_finance::Bar, &'a yahoo_finance::Bar),
    volume : u32,

}


#[tokio::main]
async fn main() {

    let my_tickers = vec!["AAPL"];

    let signals = vec![
        TimeSignal {
            name: "AAPL".to_string(),
            value: 2,
            trigger : Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        },

        TimeSignal {
            name: "AAPL".to_string(),
            value: 2,
            trigger : Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
        },

        TimeSignal {
            name: "MSFT".to_string(),
            value: 2,
            trigger : Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
        }
    ];

    let signal_groups = signals.iter().group_by(|s| &s.name);

    for  (group_name, signals) in &signal_groups {
        let data = yahoo_finance::history
            ::retrieve_interval(&group_name, yahoo_finance::Interval::_6mo).await.unwrap();

        let daytoday_direct = data.iter().clone().zip(data.iter().skip(1));
        let ordered_signals = signals.sorted_by(|a, b| a.trigger.cmp(&b.trigger)).peekable();

        let mut daytoday = daytoday_direct.clone()
            .scan((0u32, ordered_signals), |(cur_vol, next_sigs), (a, b)| {
                let midpoint_timestamp = (b.bar.timestamp + a.bar.timestamp) / 2;
                if let Some(next_sig) = next_sigs.peek() {
                    if midpoint_timestamp > next_sig.trigger.timestamp() {
                        *cur_vol += next_sig.value;
                        println!("Bought {} {} at {} on {}. Holding {}", next_sig.value, next_sig.name, a.bar.close, next_sig.trigger, cur_vol);
                        next_sigs.next();
                        
                    }
                }

                Some(Day2Day {
                    bars : (a, b),
                    volume : *cur_vol,
                })
            });
        
        let earnings = daytoday.map(|current| -> f64 {
            (current.bars.1.adjusted_close.ln() - current.bars.0.adjusted_close.ln()) * current.volume as f64
        }).sum::<f64>().exp();

        println!("Earnings for {} : {}", group_name, earnings);
        
    }
    
}



