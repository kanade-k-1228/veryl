use crate::Model;
use crate::hooks::Hook;
use std::collections::HashMap;

// シミュレータ
// model をクロックに従い時間発展させていきます
pub struct Simulator {
    model: Model, // シミュレート対象のモデル

    clock_intervals: HashMap<String, u64>, // クロック入力信号名と周期 [ns]

    simulation_time_ns: u64,                     // 現在のシミュレーション時間
    time_to_next_clock_ns: HashMap<String, u64>, // 次のクロックまでの残り時間
    clock_states: HashMap<String, bool>,         // クロックの現在の状態 (High/Low)

    hooks: Vec<Box<dyn Hook>>, // 登録されたフック
}

impl Simulator {
    pub fn new(model: Model, clocks: HashMap<String, u64>) -> Self {
        // 各クロックの次の立ち上がりまでの時間を初期化（周期の半分）
        let mut time_to_next_clock_ns = HashMap::new();
        let mut clock_states = HashMap::new();

        for (clock_name, interval) in &clocks {
            // 最初は Low から始まり、周期の半分で High になる
            time_to_next_clock_ns.insert(clock_name.clone(), interval / 2);
            clock_states.insert(clock_name.clone(), false);
        }

        Simulator {
            model,
            clock_intervals: clocks,
            simulation_time_ns: 0,
            time_to_next_clock_ns,
            clock_states,
            hooks: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.simulation_time_ns = 0;

        // クロック状態をリセット
        for (clock_name, _) in &self.clock_intervals {
            self.clock_states.insert(clock_name.clone(), false);
            // リセット後、次のクロックまでの時間を周期の半分に設定
            let interval = self.clock_intervals[clock_name];
            self.time_to_next_clock_ns
                .insert(clock_name.clone(), interval / 2);
        }

        // モデルをリセット
        self.model.reset();

        // フックに通知
        for hook in &mut self.hooks {
            hook.on_reset(self.simulation_time_ns, &self.model);
        }
    }

    /// Run simulation for specified duration in nanoseconds
    pub fn run(&mut self, duration_ns: u64) {
        let end_time = self.simulation_time_ns + duration_ns;

        while self.simulation_time_ns < end_time {
            self.step();
        }

        // シミュレーション終了をフックに通知
        for hook in &mut self.hooks {
            hook.on_finish(self.simulation_time_ns, &self.model);
        }
    }

    fn step(&mut self) {
        // 次のクロックイベントまでの最小時間を探す
        let mut min_time_to_next = u64::MAX;
        let mut next_clock = String::new();

        for (clock_name, time_to_next) in &self.time_to_next_clock_ns {
            if *time_to_next < min_time_to_next {
                min_time_to_next = *time_to_next;
                next_clock = clock_name.clone();
            }
        }

        // 時間が見つからない場合は終了
        if min_time_to_next == u64::MAX {
            return;
        }

        // シミュレーション時間を進める
        self.simulation_time_ns += min_time_to_next;

        // すべてのクロックの残り時間を更新
        for (_clock_name, time_to_next) in self.time_to_next_clock_ns.iter_mut() {
            *time_to_next -= min_time_to_next;
        }

        // ステップフックを呼ぶ
        for hook in &mut self.hooks {
            hook.on_step(self.simulation_time_ns, &self.model);
        }

        // クロックイベントを処理
        if !next_clock.is_empty() {
            let current_state = self.clock_states[&next_clock];
            let new_state = !current_state;
            self.clock_states.insert(next_clock.clone(), new_state);

            // クロックの立ち上がりエッジの場合
            if new_state {
                // pre_clockフックを呼ぶ
                for hook in &mut self.hooks {
                    hook.pre_clock(self.simulation_time_ns, &next_clock, &self.model);
                }

                // モデルのクロックを進める
                self.model.clock();

                // post_clockフックを呼ぶ
                for hook in &mut self.hooks {
                    hook.post_clock(self.simulation_time_ns, &next_clock, &self.model);
                }
            }

            // 次のクロックイベントまでの時間を設定（周期の半分）
            let interval = self.clock_intervals[&next_clock];
            self.time_to_next_clock_ns.insert(next_clock, interval / 2);
        }
    }

    /// Add a hook to the simulator
    pub fn add_hook(&mut self, hook: Box<dyn Hook>) {
        self.hooks.push(hook);
    }
}
