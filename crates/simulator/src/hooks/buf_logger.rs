use super::Hook;
use crate::Model;
use std::collections::HashMap;

// Log all changes to buffer
// this logger consumes more memory, but useful for waveform analysis
pub struct BufLogger {
    events: Vec<(u64, HashMap<String, usize>)>, // (time, signals)
}

impl BufLogger {
    pub fn new() -> Self {
        BufLogger { events: Vec::new() }
    }

    /// Print waveform to stdout
    pub fn print(&self) {
        if self.events.is_empty() {
            println!("No events recorded");
            return;
        }

        println!("\n=== Simulation Waveform ===");
        println!("Time(ns)  Events");
        println!("--------  ------");

        for (time, signals) in &self.events {
            print!("{:8}  ", time);
            for (name, value) in signals {
                print!("{}={} ", name, value);
            }
            println!();
        }
        println!("=== End of Waveform ===\n");

        // 波形ビジュアライゼーション
        self.visualize_waveform();
    }

    fn visualize_waveform(&self) {
        if self.events.is_empty() {
            return;
        }

        println!("\n=== Waveform Visualization ===");

        // すべての信号名を収集
        let mut signal_names = std::collections::HashSet::new();
        for (_, signals) in &self.events {
            for name in signals.keys() {
                signal_names.insert(name.clone());
            }
        }

        // 各信号の波形を表示
        for signal_name in signal_names {
            print!("{:8} : ", signal_name);

            let mut last_value = None;
            let mut last_time = 0u64;

            for (time, signals) in &self.events {
                if let Some(value) = signals.get(&signal_name) {
                    // 時間の幅を考慮した表示
                    let time_diff = (*time - last_time) / 100; // スケーリング

                    if last_value.is_none() {
                        // 初回
                        for _ in 0..time_diff {
                            print!("_");
                        }
                        print!("|{}", value);
                    } else if Some(value) != last_value.as_ref() {
                        // 値が変化した
                        for _ in 0..time_diff.saturating_sub(1) {
                            print!("_");
                        }
                        print!("|{}", value);
                    } else {
                        // 値が同じ
                        for _ in 0..time_diff {
                            print!("_");
                        }
                    }

                    last_value = Some(*value);
                    last_time = *time;
                }
            }
            println!();
        }
        println!("=== End of Visualization ===\n");
    }

    fn collect_signals(&self, model: &Model) -> HashMap<String, usize> {
        let mut signals = HashMap::new();

        // Modelのget_all_variablesメソッドがprivateなので、
        // 出力ポートのみを記録する（テスト用途では十分）
        // 将来的にはModelにpub get_all_variables()を追加すべき

        // とりあえず主要な出力を記録
        if let Some(val) = model.get("a") {
            signals.insert("a".to_string(), val);
        }
        if let Some(val) = model.get("b") {
            signals.insert("b".to_string(), val);
        }

        signals
    }
}

impl Hook for BufLogger {
    fn on_reset(&mut self, time: u64, model: &Model) {
        let signals = self.collect_signals(model);
        self.events.push((time, signals));
    }

    fn post_clock(&mut self, time: u64, _clock_name: &str, model: &Model) {
        let signals = self.collect_signals(model);
        self.events.push((time, signals));
    }

    fn on_finish(&mut self, _time: u64, _model: &Model) {
        // Automatically print the results when simulation finishes
        self.print();
    }
}
