require "pp"

TRUTH_TABLE = {
  "opq" => "ffff",
  "xpq" => "ffft",
  "mpq" => "fftf",
  "fpq" => "fftt",
  "lpq" => "ftff",
  "gpq" => "ftft",
  "jpq" => "fttf",
  "dpq" => "fttt",
  "kpq" => "tfff",
  "epq" => "tfft",
  "hpq" => "tftf",
  "cpq" => "tftt",
  "llpq" => "ttff",
  "bpq" => "ttft",
  "apq" => "tttf",
  "vpq" => "tttt"
}

ALIASES = {
  "false" => "opq",
  "true" => "vpq",
  "nor" => "xpq",
  "xor" => "jpq",
  "nand" => "dpq",
  "and" => "kpq",
  "xnor" => "epq",
  "or" => "apq",

  "p" => "lpq",
  "q" => "hpq",
  "pneg" => "fpq",
  "qneg" => "gpq",

  "cnon" => "mpq",
  "mnon" => "lpq",
  "minpl" => "cpq",
  "cinpl" => "bpq"
}

def withalias(e)
  spc = false
  if ALIASES.include? e then
    spc = true
    print("#{e} = ")
    e = ALIASES[e]
    puts e
  end
  if TRUTH_TABLE.include? e then
    spc = true
    print("#{e} = ")
    e = TRUTH_TABLE[e]
    puts e
  end
  puts if spc
  return e
end

def withtruth(e)
  if "t1".include? e or e == "true" then
    return true
  elsif "f0".include? e or e == "false" then
    return false
  else
    raise ArgumentError
  end
end

def genvalues(inputs)
  vaules = []
  0.upto(2**inputs-1) { |ic_|
    ic = ic_
    values = []
    0.upto(inputs-1) { |ac|
      values << ((ic >> ac) % 2 == 1)
    }
    vaules << values.reverse
  }
  return vaules.reverse
end

def v(str)
  str.split("").map { |e| withtruth(e) }
end

def evalfun(values, yv)
  gen = genvalues(Math.log(values.length,2))
  gen.each_with_index { |val,i| if val == yv then return values[i] end }
  return nil
end

def combfun(values1,values2)
  vi1 = Math.log(values1.length,2)
  vi2 = Math.log(values2.length,2)
  gen = genvalues(vi1+vi2-1)
  retv = []
  gen.each_with_index { |g,i|
    arg1 = g.pop(vi1)
    g << evalfun(values1,arg1)
    arg2 = g.pop(vi2)
    retv << evalfun(values2,arg2)
  }
  return retv
end

def printfun(values)
  s = ""
  values.each { |v| if v then s << "t" else s << "f" end }
  return s.split("").reverse.each_slice(4).to_a.map{ |e| e.reverse.join("") }.reverse.join(".")
end

def evalexpr(str)
  str.delete!(".")
  str.delete!(",")
  if str.include?("-") then
    sv, str = str.split("-")
    val = sv.split(" ").map { |e| withtruth(e) }
  end
  if str.length < 1 then
    return
  end
  if str == "?" then
    PP.pp(TRUTH_TABLE)
    PP.pp(ALIASES)
    puts "true\nfalse\n"
    return
  end
  fun = str.split(" ")
    .map { |e| v(withalias(e)) }.reduce { |e,u| combfun(e, u) }
  puts(printfun(fun))
  if val then
    res = evalfun(fun, val)
    puts res
  end
end

loop do
  print "$ "
  begin
    evalexpr(STDIN.readline.chomp)
  rescue ArgumentError
    puts "?"
  end
end
