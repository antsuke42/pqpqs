use std::collections::HashMap;
use std::io;
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
"#;

#[derive(Debug, PartialEq)]
struct Values {
    inputs: i32,
    values: Vec<Vec<bool>>,
}

impl Values {
    fn genvalues(inputs: i32) -> Values {
        let mut values = Vec::new();
        for ic in (0..2_i32.pow(inputs.try_into().unwrap())).rev() {
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
    inputs: i32,
    values: Vec<bool>,
}

impl Func {
    fn from_expr(s: &str) -> Option<Self> {
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
        let uinputs: usize = self.inputs.try_into().unwrap();
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

#[derive(Debug)]
enum Expr {
    F(Func),
}

#[derive(Debug)]
struct Expression(Vec<Expr>);

impl Expression {
    fn eval_str(spx: &str, dict: &Dictionary) -> Option<Expression> {
        let mut v = Vec::new();
        for sx in spx.split_whitespace() {
            v.push(Expr::F(Func::from_expr(&dict.try_find(sx.to_owned()))?));
        }
        dbg!(&v);
        return Some(Expression(v));
    }
    fn count_inputs(&self) -> i32 {
        let mut i = 1;
        for sx in &self.0 {
            match sx {
                Expr::F(f) => i = i + f.inputs - 1,
            }
        }
        return i;
    }
    fn eval_expr(&self, mut stack: Stack) -> Option<Stack> {
        for sx in &self.0 {
            match sx {
                Expr::F(f) => {
                    let mut res = f.eval(&mut stack)?;
                    stack.append(&mut res);
                }
            }
        }
        return Some(stack);
    }
}

type Stack = Vec<bool>;

fn fmt_stack(stack: &Vec<bool>) -> String {
    let mut st = String::new();
    let mut s = false;
    for b in stack {
        st.push_str(&b.to_string());
        if s {
            st.push_str(" ");
            s = true;
        }
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
        let mut c = false;
        loop {
            match self.0.get(&qa) {
                Some(v) => {
                    println!("{} = {}", qa, v);
                    c = true;
                    qa = v.to_owned();
                }
                None => {
                    if c {
                        println!("");
                    }
                    return qa;
                }
            }
        }
    }
}

pub fn repl() {
    let dictionary = Dictionary::new_stdenv();
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("$ ");
        stdout.flush().unwrap();
        buffer.clear();
        let _u = stdin.read_line(&mut buffer).unwrap();
        match Expression::eval_str(&buffer, &dictionary) {
            Some(expr) => match expr.eval_expr(Stack::new()) {
                Some(stack) => {
                    println!("{}", fmt_stack(&stack));
                    continue;
                }
                _ => {
                    let input = expr.count_inputs();
                    dbg!(input);
                    let values = Values::genvalues(input);
                    let mut res = Vec::new();
                    for v in values.values {
                        res.push(expr.eval_expr(v).unwrap());
                    }
                    dbg!(res);
                }
            },
            _ => println!("?"),
        }
    }
}
