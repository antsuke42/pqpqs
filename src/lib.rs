use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use std::io::Write;

const STDENV: &str = r#"
true t
false f

op ff
np ft
lp tf
vp tt

id lp
not np
neg not
lt vp
ltrue lt
lf op
lfalse lf

opq ffff
xpq ffft
mpq fftf
fpq fftt
lpq ftff
gpq ftft
jpq fttf
dpq fttt
kpq tfff
epq tfft
hpq tftf
cpq tftt
ipq ttff
bpq ttft
apq tttf
vpq tttt

cont opq
taut vpq
nor xpq
xor jpq
nand dpq
and kpq
xnor epq
or apq
p ipq
q hpq
pneg fpq
qneg gpq
impl cpq
cimpl bpq
nonimpl lpq
cnonimpl mpq

drop -
dup -'.'
over --'.''.'
swap --''.'
rot ---''.'''.'

2drop --
2dup --'.''.'.''
2over ----'.''.'''.''''.'.''
2swap ----'''.''''.'.''
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
    values: Vec<usize>,
}

impl StackC {
    fn from_str(s: &str) -> Option<StackC> {
        let inputs = s.matches("-").count();
        if inputs <= 0 || inputs + s.matches(".").count() + s.matches("'").count() != s.len() {
            return None;
        }
        let es = &s[inputs..];
        let mut values = Vec::new();
        for u in es.split(".") {
            if u.len() > 0 {
                values.push(u.len() - 1);
            }
        }
        return Some(StackC {
            inputs,
            outputs: values.len(),
            values,
        });
    }
    fn eval(&self, stack: &mut Stack) -> Option<Stack> {
        let mut ret = Vec::new();
        let uinputs: usize = self.inputs;
        if stack.len() < uinputs {
            return None;
        }
        let input = stack.split_off(stack.len() - uinputs);
        for val in &self.values {
            ret.push(input[val - 0])
        }
        return Some(ret);
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
    fn count_inputs(&self) -> usize {
        let mut i: i32 = 0;
        let mut mi: i32 = 0;
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
            if i > mi {
                mi = i;
            }
            i = i - outputs;
        }
        return mi.try_into().unwrap();
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
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("> {}", line);
        if line == "?" {
            println!("{}", STDENV);
        }
        match Expression::eval_str(&line, &dictionary) {
            Some(expr) => match expr.eval_expr(Stack::new()) {
                Some(stack) => {
                    println!("{}", fmt_stack(&stack));
                }
                _ => {
                    let input = expr.count_inputs();
                    dbg!(&expr);
                    dbg!(&input);
                    let values = Values::genvalues(input);
                    let mut res = Vec::new();
                    for v in values.values {
                        match expr.eval_expr(v) {
                            Some(e) => res.push(e[0]),
                            None => println!("...?"),
                        }
                    }
                    println!("{}", fmt_func(&res));
                }
            },
            _ => println!("?"),
        }
        stdout.flush().unwrap();
    }
}
