use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;

const STDENV: &str = r#"
help ?

true t
false f

lf ff
not ft
id tf
lt tt

cont ffff
nor ffft
cnonimpl fftf
pneg fftt
nonimpl ftff
qneg ftft
xor fttf
nand fttt
and tfff
xnor tfft
q tftf
impl tftt
p ttff
cimpl ttft
or tttf
taut tttt

drop #

dup '
over '-
2dup ''
2over ''--

swap -*
2swap --**
rot -**
"#;

#[derive(Debug, PartialEq)]
struct Values {
    inputs: usize,
    values: Vec<Vec<bool>>,
}

impl Values {
    fn genvalues(inputs: usize) -> Values {
        let mut values = Vec::new();
        for ic in (0..2_usize.pow(inputs.try_into().unwrap())).rev() {
            let mut vaules = Vec::new();
            for ac in (0..inputs).rev() {
                vaules.push((ic >> ac) % 2 == 1);
            }
            values.push(vaules);
        }
        return Values { inputs, values };
    }
}

fn asbool(s: &char) -> Option<bool> {
    return match s {
        't' => Some(true),
        '1' => Some(true),
        'f' => Some(false),
        '0' => Some(false),
        _ => None,
    };
}

#[derive(Debug, PartialEq)]
struct Func {
    inputs: usize,
    values: Vec<bool>,
}

impl Func {
    fn from_str(s: &str) -> Option<Self> {
        let mut values = Vec::new();
        let mut u = s.len();
        let mut inputs = 0;
        while u > 1 {
            if u % 2 != 0 {
                return None;
            }
            inputs = inputs + 1;
            u = u / 2;
        }
        for c in s.chars() {
            let b = asbool(&c);
            match b {
                Some(l) => values.push(l),
                None => return None,
            }
        }
        return Some(Func { inputs, values });
    }
    fn eval(&self, stack: &mut Stack) -> Option<Stack> {
        let gen = Values::genvalues(self.inputs);
        let mut ret = Vec::new();
        let uinputs: usize = self.inputs;
        if stack.len() < uinputs {
            return None;
        }
        let input = stack.split_off(stack.len() - uinputs);
        for (i, val) in gen.values.iter().enumerate() {
            if *val == input {
                ret.push(self.values[i]);
            }
        }
        return Some(ret);
    }
}

fn fmt_func(values: &Vec<bool>) -> String {
    let mut st = String::new();
    let mut i: usize = 0;
    for val in values {
        i += 1;
        if i == 5 {
            st.push('.');
            i = 0;
        }

        if *val {
            st.push('t');
        } else {
            st.push('f');
        }
    }
    return st;
}

#[derive(Debug, PartialEq)]
struct StackC {
    inputs: usize,
    outputs: usize,
    values: Vec<SC>,
}

#[derive(Debug, PartialEq)]
enum SC {
    Take,
    Over,
    Swap,
    Drop,
}

impl StackC {
    fn from_str(s: &str) -> Option<StackC> {
        let inputs = s.len();
        if inputs <= 0 || s.chars().filter(|&x| "-'*#".contains(x)).count() != inputs {
            return None;
        }
        let mut values = Vec::new();
        let mut outputs = 0;
        for u in s.chars() {
            values.push(match u {
                '-' => {
                    outputs += 1;
                    SC::Take
                }
                '\'' => {
                    outputs += 2;
                    SC::Over
                }
                '*' => {
                    outputs += 1;
                    SC::Swap
                }
                '#' => SC::Drop,
                _ => {
                    return None;
                }
            })
        }
        return Some(StackC {
            inputs,
            outputs,
            values,
        });
    }
    fn eval(&self, stack: &mut Stack) -> Option<Stack> {
        let uinputs: usize = self.inputs;
        if stack.len() < uinputs {
            return None;
        }
        let input = stack.split_off(stack.len() - uinputs);

        let mut main = Vec::new();
        let mut over = Vec::new();
        let mut swap = Vec::new();

        for (i, val) in self.values.iter().enumerate() {
            match val {
                SC::Take => {
                    main.push(input[i]);
                }
                SC::Over => {
                    main.push(input[i]);
                    over.push(input[i]);
                }
                SC::Swap => {
                    swap.push(input[i]);
                }
                SC::Drop => {}
            }
        }
        swap.append(&mut main);
        swap.append(&mut over);

        return Some(swap);
    }
}

#[derive(Debug)]
enum Expr {
    F(Func),
    C(StackC),
}

#[derive(Debug)]
struct Expression(Vec<Expr>);

impl Expression {
    fn eval_str(spx: &str, dict: &Dictionary) -> Option<Expression> {
        let mut v = Vec::new();
        for sx in spx.split_whitespace() {
            let word = dict.try_find(sx.to_owned());
            let f = Func::from_str(&word).map(Expr::F);
            let c = StackC::from_str(&word).map(Expr::C);
            v.push(f.or(c)?);
        }
        return Some(Expression(v));
    }
    fn count_io(&self) -> (usize, usize) {
        let mut i: i32 = 0;
        let mut mi: i32 = 0;
        let mut o: i32 = 0;
        for sx in &self.0 {
            let inputs: i32;
            let outputs: i32;
            match sx {
                Expr::F(f) => {
                    inputs = f.inputs.try_into().unwrap();
                    outputs = 1;
                }
                Expr::C(f) => {
                    inputs = f.inputs.try_into().unwrap();
                    outputs = f.outputs.try_into().unwrap();
                }
            }
            i = i + inputs;
            o = o - inputs;
            if o < 0 {
                o = 0;
            }
            if i > mi {
                mi = i;
            }
            i = i - outputs;
            o = o + outputs;
        }
        return (mi.try_into().unwrap(), o.try_into().unwrap());
    }
    fn eval_expr(&self, mut stack: Stack) -> Option<Stack> {
        for sx in &self.0 {
            let mut res;
            match sx {
                Expr::F(f) => {
                    res = f.eval(&mut stack)?;
                }
                Expr::C(f) => {
                    res = f.eval(&mut stack)?;
                }
            }
            stack.append(&mut res);
        }
        return Some(stack);
    }
}

type Stack = Vec<bool>;

fn fmt_stack(stack: &Vec<bool>) -> String {
    let mut st = String::new();
    let mut s = false;
    for b in stack {
        if s {
            st.push_str(" ");
        }
        s = true;
        st.push_str(&b.to_string());
    }
    return st;
}

#[derive(Debug)]
struct Dictionary(HashMap<String, String>);

impl Dictionary {
    fn new_stdenv() -> Self {
        let mut m = HashMap::new();
        for sd in STDENV.split("\n") {
            let a: Vec<&str> = sd.split_whitespace().collect();
            if a.len() == 2 {
                m.insert(a[0].to_string(), a[1].to_string());
            }
        }
        return Dictionary(m);
    }
    fn try_find(&self, q: String) -> String {
        let mut qa = q;
        loop {
            match self.0.get(&qa) {
                Some(v) => {
                    println!("({} = {})", qa, v);
                    qa = v.to_owned();
                }
                None => {
                    return qa;
                }
            }
        }
    }
}

pub fn repl() {
    let dictionary = Dictionary::new_stdenv();

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                eval(&line, &dictionary);
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("{:?}", err);
                break;
            }
        }
    }
}

fn eval(line: &str, dictionary: &Dictionary) {
    if line == "?" || line == "help" {
        println!("{}", STDENV);
        return;
    }
    match Expression::eval_str(&line, &dictionary) {
        Some(expr) => {
            let (input, o) = expr.count_io();
            if input == 0 {
                match expr.eval_expr(Stack::new()) {
                    Some(stack) => println!("{}", fmt_stack(&stack)),
                    None => println!("???"),
                }
            } else {
                if o != 1 {
                    println!("... ({})", o);
                    return;
                }
                let values = Values::genvalues(input);
                let mut res = Vec::new();
                for v in values.values {
                    match expr.eval_expr(v) {
                        Some(e) => res.push(e[0]),
                        None => {
                            println!("...?");
                            return;
                        }
                    }
                }
                println!("{}", fmt_func(&res));
            }
        }
        _ => println!("?"),
    }
}
