use std::collections::HashMap;
use veryl_analyzer::symbol::SymbolKind;
use veryl_analyzer::{definition_table, symbol_table};
use veryl_parser::ParolError;
use veryl_parser::veryl_grammar_trait::{self as syntax_tree, VerylGrammarTrait};
use veryl_parser::veryl_walker::{Handler, HandlerPoint, VerylWalker};

// 代入式を表す構造体
#[derive(Debug, Clone)]
pub struct Assignment {
    target: String,   // 代入先の信号名
    expression: Expr, // 代入する式
}

// 式を表す列挙型
#[derive(Debug, Clone)]
pub enum Expr {
    Const(usize),              // 定数値
    Var(String),               // 変数参照
    Add(Box<Expr>, Box<Expr>), // 加算
    Sub(Box<Expr>, Box<Expr>), // 減算
    Mul(Box<Expr>, Box<Expr>), // 乗算
    Div(Box<Expr>, Box<Expr>), // 除算
    Not(Box<Expr>),            // ビット反転
}

impl Expr {
    pub fn eval(&self, env: &HashMap<String, usize>) -> usize {
        match self {
            Expr::Const(val) => *val,
            Expr::Var(name) => {
                // 変数の値を取得（見つからない場合は0）
                env.get(name).copied().unwrap_or(0)
            }
            Expr::Add(left, right) => left.eval(env) + right.eval(env),
            Expr::Sub(left, right) => left.eval(env).saturating_sub(right.eval(env)),
            Expr::Mul(left, right) => left.eval(env) * right.eval(env),
            Expr::Div(left, right) => {
                let right_val = right.eval(env);
                if right_val != 0 {
                    left.eval(env) / right_val
                } else {
                    0 // ゼロ除算を回避
                }
            }
            Expr::Not(expr) => {
                let val = expr.eval(env);
                // ビット反転（値が0なら1、それ以外なら0にする）
                // これによりトグルフリップフロップのような動作になる
                if val == 0 { 1 } else { 0 }
            }
        }
    }
}

// 順序回路のブロック（always_ff）
#[derive(Debug, Clone)]
pub struct SequentialBlock {
    reset_assignments: Vec<Assignment>, // リセット時の代入
    clock_assignments: Vec<Assignment>, // クロック時の代入
}

// ASTから代入式を収集するハンドラ
struct AssignCollector {
    assignments: Vec<Assignment>,
    sequential_blocks: Vec<SequentialBlock>,
    in_always_ff: bool,
    current_sequential: Option<SequentialBlock>,
    handler_point: HandlerPoint,
}

impl AssignCollector {
    fn new() -> Self {
        Self {
            assignments: Vec::new(),
            sequential_blocks: Vec::new(),
            in_always_ff: false,
            current_sequential: None,
            handler_point: HandlerPoint::Before,
        }
    }

    // Expressionを評価してExprに変換
    fn convert_expression(&self, expr: &syntax_tree::Expression) -> Expr {
        self.convert_expression01(&expr.if_expression.expression01)
    }

    fn convert_expression01(&self, expr: &syntax_tree::Expression01) -> Expr {
        // 今のところ、最初の項だけを処理
        self.convert_expression02(&expr.expression02)
    }

    fn convert_expression02(&self, expr: &syntax_tree::Expression02) -> Expr {
        self.convert_expression03(&expr.expression03)
    }

    fn convert_expression03(&self, expr: &syntax_tree::Expression03) -> Expr {
        self.convert_expression04(&expr.expression04)
    }

    fn convert_expression04(&self, expr: &syntax_tree::Expression04) -> Expr {
        self.convert_expression05(&expr.expression05)
    }

    fn convert_expression05(&self, expr: &syntax_tree::Expression05) -> Expr {
        self.convert_expression06(&expr.expression06)
    }

    fn convert_expression06(&self, expr: &syntax_tree::Expression06) -> Expr {
        self.convert_expression07(&expr.expression07)
    }

    fn convert_expression07(&self, expr: &syntax_tree::Expression07) -> Expr {
        self.convert_expression08(&expr.expression08)
    }

    fn convert_expression08(&self, expr: &syntax_tree::Expression08) -> Expr {
        self.convert_expression09(&expr.expression09)
    }

    fn convert_expression09(&self, expr: &syntax_tree::Expression09) -> Expr {
        // 加算・減算の処理
        let mut result = self.convert_expression10(&expr.expression10);
        for item in &expr.expression09_list {
            let right = self.convert_expression10(&item.expression10);
            // Operator10は+と-を表す
            let op_str = item.operator10.operator10_token.to_string();
            match op_str.as_str() {
                "+" => {
                    result = Expr::Add(Box::new(result), Box::new(right));
                }
                "-" => {
                    result = Expr::Sub(Box::new(result), Box::new(right));
                }
                _ => {} // その他の演算子は今のところ無視
            }
        }
        result
    }

    fn convert_expression10(&self, expr: &syntax_tree::Expression10) -> Expr {
        // 乗算・除算の処理
        let mut result = self.convert_expression11(&expr.expression11);
        for item in &expr.expression10_list {
            let right = self.convert_expression11(&item.expression11);
            // Expression10ListGroupはenumなので、パターンマッチング
            match &*item.expression10_list_group {
                syntax_tree::Expression10ListGroup::Operator11(op) => {
                    let op_str = op.operator11.operator11_token.to_string();
                    match op_str.as_str() {
                        "*" => {
                            result = Expr::Mul(Box::new(result), Box::new(right));
                        }
                        "/" => {
                            result = Expr::Div(Box::new(result), Box::new(right));
                        }
                        _ => {} // その他の演算子は今のところ無視
                    }
                }
                syntax_tree::Expression10ListGroup::Star(_) => {
                    result = Expr::Mul(Box::new(result), Box::new(right));
                }
            }
        }
        result
    }

    fn convert_expression11(&self, expr: &syntax_tree::Expression11) -> Expr {
        self.convert_expression12(&expr.expression12)
    }

    fn convert_expression12(&self, expr: &syntax_tree::Expression12) -> Expr {
        // Expression12は型キャスト用なので、そのままexpression13に委譲
        self.convert_expression13(&expr.expression13)
    }

    fn convert_expression13(&self, expr: &syntax_tree::Expression13) -> Expr {
        // 単項演算子の処理
        let mut result = self.convert_factor(&expr.factor);
        // 単項演算子を右から左に適用
        for item in expr.expression13_list.iter().rev() {
            match &*item.expression13_list_group {
                syntax_tree::Expression13ListGroup::UnaryOperator(unary_op) => {
                    let op_str = unary_op.unary_operator.unary_operator_token.to_string();
                    match op_str.as_str() {
                        "~" => {
                            result = Expr::Not(Box::new(result));
                        }
                        _ => {} // その他の単項演算子は今のところ無視
                    }
                }
                _ => {} // その他の演算子グループは今のところ無視
            }
        }
        result
    }

    fn extract_assignment_from_statement_block(
        &self,
        statement_list: &syntax_tree::StatementBlockList,
    ) -> Option<Assignment> {
        // StatementBlockListからAssignmentを抽出
        // StatementBlockGroupを処理
        match &*statement_list
            .statement_block_group
            .statement_block_group_group
        {
            syntax_tree::StatementBlockGroupGroup::StatementBlockItem(item) => {
                match &*item.statement_block_item {
                    syntax_tree::StatementBlockItem::Statement(stmt) => {
                        if let syntax_tree::Statement::IdentifierStatement(id_stmt) =
                            &*stmt.statement
                        {
                            let stmt = &id_stmt.identifier_statement;

                            // 識別子から代入先を取得
                            let target = match &*stmt.expression_identifier.scoped_identifier.scoped_identifier_group {
                                syntax_tree::ScopedIdentifierGroup::IdentifierScopedIdentifierOpt(id_group) => {
                                    id_group.identifier.identifier_token.to_string()
                                }
                                _ => return None,
                            };

                            // IdentifierStatementGroupから代入の右辺を取得
                            match &*stmt.identifier_statement_group {
                                syntax_tree::IdentifierStatementGroup::Assignment(a) => {
                                    let expression =
                                        self.convert_expression(&a.assignment.expression);
                                    Some(Assignment { target, expression })
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn extract_assignment_from_statement(
        &self,
        statement: &syntax_tree::IfResetStatementList,
    ) -> Option<Assignment> {
        // IfResetStatementListはstatement_blockを持つ
        // StatementBlockを再帰的に処理
        for statement_block_item in &statement.statement_block.statement_block_list {
            if let Some(assignment) =
                self.extract_assignment_from_statement_block(statement_block_item)
            {
                return Some(assignment);
            }
        }
        None
    }

    fn convert_factor(&self, factor: &syntax_tree::Factor) -> Expr {
        match factor {
            syntax_tree::Factor::IdentifierFactor(f) => {
                // 識別子の処理
                // ScopedIdentifierはenumなので、パターンマッチング
                match &*f
                    .identifier_factor
                    .expression_identifier
                    .scoped_identifier
                    .scoped_identifier_group
                {
                    syntax_tree::ScopedIdentifierGroup::IdentifierScopedIdentifierOpt(id_group) => {
                        let id = id_group.identifier.identifier_token.to_string();
                        Expr::Var(id)
                    }
                    _ => Expr::Const(0), // その他の形式は今のところ0として扱う
                }
            }
            syntax_tree::Factor::Number(n) => {
                // 数値リテラルの処理
                match &*n.number {
                    syntax_tree::Number::IntegralNumber(integral) => {
                        match &*integral.integral_number {
                            syntax_tree::IntegralNumber::Based(based) => {
                                // 基数指定の数値（例：32'h10）
                                let s = based.based.based_token.to_string();
                                // 基数指定のフォーマット（例：32'h10）をパース
                                if let Some(pos) = s.rfind('\'') {
                                    let num_part = &s[pos + 2..]; // 'h' や 'b' の後の部分
                                    let base = match s.chars().nth(pos + 1) {
                                        Some('h') | Some('H') => 16,
                                        Some('d') | Some('D') => 10,
                                        Some('b') | Some('B') => 2,
                                        Some('o') | Some('O') => 8,
                                        _ => 10,
                                    };
                                    if let Ok(val) = usize::from_str_radix(num_part, base) {
                                        Expr::Const(val)
                                    } else {
                                        Expr::Const(0)
                                    }
                                } else {
                                    Expr::Const(0)
                                }
                            }
                            syntax_tree::IntegralNumber::BaseLess(baseless) => {
                                // 単純な10進数
                                let s = baseless.base_less.base_less_token.to_string();
                                if let Ok(val) = s.parse::<usize>() {
                                    Expr::Const(val)
                                } else {
                                    Expr::Const(0)
                                }
                            }
                            _ => Expr::Const(0), // その他の形式は今のところ0として扱う
                        }
                    }
                    _ => Expr::Const(0), // RealNumberなどは今のところ0として扱う
                }
            }
            _ => Expr::Const(0), // その他のFactorは今のところ0として扱う
        }
    }
}

impl VerylWalker for AssignCollector {
    fn get_handlers(&mut self) -> Option<Vec<(bool, &mut dyn Handler)>> {
        Some(vec![(true, self as &mut dyn Handler)])
    }
}

impl VerylGrammarTrait for AssignCollector {
    fn assign_declaration(
        &mut self,
        arg: &syntax_tree::AssignDeclaration,
    ) -> Result<(), ParolError> {
        // 代入先の取得
        let target = match &*arg.assign_destination {
            syntax_tree::AssignDestination::HierarchicalIdentifier(h) => h
                .hierarchical_identifier
                .identifier
                .identifier_token
                .to_string(),
            _ => return Ok(()), // 他の形式は今のところ無視
        };

        // 式の変換
        let expression = self.convert_expression(&arg.expression);

        // 代入式を追加
        self.assignments.push(Assignment { target, expression });

        Ok(())
    }

    fn always_ff_declaration(
        &mut self,
        _arg: &syntax_tree::AlwaysFfDeclaration,
    ) -> Result<(), ParolError> {
        // HandlerPoint::Beforeの場合はalways_ffブロックの開始
        if matches!(self.handler_point, HandlerPoint::Before) {
            self.in_always_ff = true;
            self.current_sequential = Some(SequentialBlock {
                reset_assignments: Vec::new(),
                clock_assignments: Vec::new(),
            });
        }
        // HandlerPoint::Afterの場合はalways_ffブロックの終了
        else if matches!(self.handler_point, HandlerPoint::After) {
            self.in_always_ff = false;
            if let Some(block) = self.current_sequential.take() {
                self.sequential_blocks.push(block);
            }
        }
        Ok(())
    }

    fn if_reset_statement(
        &mut self,
        arg: &syntax_tree::IfResetStatement,
    ) -> Result<(), ParolError> {
        if !self.in_always_ff {
            return Ok(());
        }

        // HandlerPoint::Beforeの場合は直接if_resetの内容を処理
        if matches!(self.handler_point, HandlerPoint::Before) {
            // if_resetブロックの処理（リセット時の代入）
            for statement in &arg.if_reset_statement_list {
                if let Some(assignment) = self.extract_assignment_from_statement(statement) {
                    if let Some(ref mut block) = self.current_sequential {
                        block.reset_assignments.push(assignment);
                    }
                }
            }

            // else節の処理（クロック時の代入）
            if let Some(ref else_clause) = arg.if_reset_statement_opt {
                for statement_block_item in &else_clause.statement_block.statement_block_list {
                    if let Some(assignment) =
                        self.extract_assignment_from_statement_block(statement_block_item)
                    {
                        if let Some(ref mut block) = self.current_sequential {
                            block.clock_assignments.push(assignment);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn identifier_statement(
        &mut self,
        _arg: &syntax_tree::IdentifierStatement,
    ) -> Result<(), ParolError> {
        // if_reset_statementで直接処理するので、ここでは何もしない
        // これにより重複を避ける
        Ok(())
    }
}

impl Handler for AssignCollector {
    fn set_point(&mut self, p: HandlerPoint) {
        self.handler_point = p;
    }
}

// Model は module のシミュレーションモデルを表します
pub struct Model {
    // モジュール名
    _module_name: String,

    // クロック
    _clocks: Vec<String>,

    // リセット
    _resets: Vec<String>,

    // 入力ポート
    inputs: HashMap<String, usize>,

    // 出力ポート
    outputs: HashMap<String, usize>,

    // 内部信号
    internals: HashMap<String, usize>,

    // 組み合わせ回路の式（assign文など）
    combinational: Vec<Assignment>,

    // 順序回路ブロック（always_ff）
    sequential: Vec<SequentialBlock>,

    // リセット中かどうか
    is_reset: bool,
}

impl Model {
    pub fn new(top: &str, init: HashMap<String, usize>) -> Self {
        // シミュレーションに必要な情報をsymbol_tableから収集する
        let mut inputs = HashMap::new();
        let mut outputs = HashMap::new();
        let internals = HashMap::new();
        let mut combinational = Vec::new();
        let mut sequential = Vec::new();
        let mut clocks = Vec::new();
        let mut resets = Vec::new();

        // symbol_tableからモジュールを検索
        for symbol in symbol_table::get_all() {
            if let SymbolKind::Module(m) = &symbol.kind {
                if symbol.token.to_string() == top {
                    // ポート情報を取得
                    for port in &m.ports {
                        if let Some(port_symbol) = symbol_table::get(port.symbol) {
                            if let SymbolKind::Port(p) = &port_symbol.kind {
                                let port_name = port_symbol.token.to_string();

                                // 入力/出力ポートを分類
                                match p.direction {
                                    veryl_analyzer::symbol::Direction::Input => {
                                        // 初期値がinitで指定されていればそれを使用
                                        let initial_value =
                                            init.get(&port_name).copied().unwrap_or(0);
                                        inputs.insert(port_name.clone(), initial_value);

                                        // クロック、リセット信号を識別
                                        // TypeのDebug出力を使用して判定
                                        let type_str = format!("{:?}", p.r#type);
                                        if type_str.contains("Clock") {
                                            clocks.push(port_name);
                                        } else if type_str.contains("Reset") {
                                            resets.push(port_name);
                                        }
                                    }
                                    veryl_analyzer::symbol::Direction::Output => {
                                        outputs.insert(port_name, 0);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    // definition_tableからモジュールの定義を取得してASTを解析
                    if let Some(definition) = definition_table::get(m.definition) {
                        if let veryl_analyzer::definition_table::Definition::Module(module_decl) =
                            definition
                        {
                            // AssignCollectorを使ってassign文とalways_ffブロックを収集
                            let mut collector = AssignCollector::new();

                            // モジュール全体をトラバースする
                            VerylWalker::module_declaration(&mut collector, &module_decl);

                            // 収集した代入式を追加
                            combinational = collector.assignments;
                            sequential = collector.sequential_blocks;
                        }
                    }
                }
            }
        }

        let mut model = Self {
            _module_name: top.to_string(),
            inputs,
            outputs,
            internals,
            combinational,
            sequential,
            _clocks: clocks,
            _resets: resets,
            is_reset: false,
        };

        // 初期評価（組み合わせ回路の評価）
        model.evaluate_combinational();

        model
    }

    pub fn input(&mut self, port: &str, value: usize) {
        if self.inputs.contains_key(port) {
            self.inputs.insert(port.to_string(), value);
            // 入力が変更されたら組み合わせ回路を再評価
            self.evaluate_combinational();
        }
    }

    pub fn get(&self, port: &str) -> Option<usize> {
        self.outputs.get(port).copied()
    }

    pub fn clock(&mut self) {
        if !self.is_reset {
            // リセット中でなければ、クロックエッジで順序回路を評価
            self.evaluate_sequential_clock();
            // 順序回路の出力が変わった可能性があるので組み合わせ回路も再評価
            self.evaluate_combinational();
        }
    }

    pub fn reset(&mut self) {
        self.is_reset = true;
        // リセット時の順序回路を評価
        self.evaluate_sequential_reset();
        // リセット解除
        self.is_reset = false;
        // リセット後の組み合わせ回路を評価
        self.evaluate_combinational();
    }

    fn evaluate_combinational(&mut self) {
        for assignment in &self.combinational {
            let variables = self.get_all_variables();
            let value = assignment.expression.eval(&variables);

            // 出力ポートに値を設定
            if self.outputs.contains_key(&assignment.target) {
                self.outputs.insert(assignment.target.clone(), value);
            }
            // 内部信号に値を設定
            else if self.internals.contains_key(&assignment.target) {
                self.internals.insert(assignment.target.clone(), value);
            }
        }
    }

    fn evaluate_sequential_reset(&mut self) {
        // 全ての順序ブロックのリセット処理を実行
        for block in &self.sequential {
            for assignment in &block.reset_assignments {
                let variables = self.get_all_variables();
                let value = assignment.expression.eval(&variables);

                // 出力ポートに値を設定
                if self.outputs.contains_key(&assignment.target) {
                    self.outputs.insert(assignment.target.clone(), value);
                }
                // 内部信号に値を設定
                else if self.internals.contains_key(&assignment.target) {
                    self.internals.insert(assignment.target.clone(), value);
                }
            }
        }
    }

    fn evaluate_sequential_clock(&mut self) {
        // 全ての順序ブロックのクロック処理を実行
        for block in &self.sequential {
            for assignment in &block.clock_assignments {
                let variables = self.get_all_variables();
                let value = assignment.expression.eval(&variables);

                // 出力ポートに値を設定
                if self.outputs.contains_key(&assignment.target) {
                    self.outputs.insert(assignment.target.clone(), value);
                }
                // 内部信号に値を設定
                else if self.internals.contains_key(&assignment.target) {
                    self.internals.insert(assignment.target.clone(), value);
                }
            }
        }
    }

    /// すべての変数（入力、出力、内部信号）を一つのHashMapにまとめて返す
    fn get_all_variables(&self) -> HashMap<String, usize> {
        let mut variables = HashMap::new();

        // 入力ポートの値を追加
        for (name, value) in &self.inputs {
            variables.insert(name.clone(), *value);
        }

        // 出力ポートの値を追加
        for (name, value) in &self.outputs {
            variables.insert(name.clone(), *value);
        }

        // 内部信号の値を追加
        for (name, value) in &self.internals {
            variables.insert(name.clone(), *value);
        }

        variables
    }
}
