import { generate as pegGenerate } from 'pegjs';

interface VarPattern {
  var: string;
}

type Pattern = VarPattern;

interface LetExpr {
  let: Pattern;
  be: Expr;
  in: Expr;
}

interface DoExpr {
  do: Expr;
  in: Expr;
}

interface EffectExpr {
  effect: Expr;
  body: Expr;
}

interface AffectExpr {
  affect: Expr;
}

interface BinaryExpr {
  binary: string;
  left: Expr;
  right: Expr;
}

interface LiteralExpr {
  literal: string | number;
}

interface RefExpr {
  ref: string;
}

interface CallExpr {
  call: Expr;
  args: Expr[];
}

type Expr =
  | LiteralExpr
  | RefExpr
  | BinaryExpr
  | CallExpr
  | EffectExpr
  | AffectExpr
  | LetExpr;

interface IOAction {
  io: () => Value;
}

interface IOContext {
  actions: IOAction[];
}

const ioEffect = {
  inject: (action: IOAction): IOContext =>
    ({ actions: [action] }),
  start: (): IOContext =>
    ({ actions: [] }),
  affect: (context: IOContext, action: IOAction) =>
    ({ actions: [...context.actions, action] }),
};

type Ident = string;

interface Value {
  value: string | number | Value[] | object | IOAction;
}

interface Env {
  parent: Env | undefined;
  bindings: Map<Ident, Value>;
}

const peg = `
start
  = shebang? (_ ";")* _ expr:expr _ { return expr; }

expr
  = let
  / do

let
  = "let" _ pat:pattern _ "=" _ init:factor mulspace _ body:expr {
    return { let: pat, init, body };
  }
  / "let" _ pat:pattern _ "=" _ init:term _ body:expr {
  	return { err: "Semicolon required", let: pat, init, body };
  }
  / "let" _ pat:pattern _ "=" _ init:term (_ ";")+ _ body:expr {
  	return { let: pat, init, body };
  }
  / "let" _ pat:pattern _ "=" _ init:term (_ ";")* {
    return { let: pat, init, body: { literal: 0 } };
  }
  / "let" _ pat:pattern _ "=" _ ";" _ body:expr {
    return { err: "Missing initializer", pat, body };
  }

do
  = init:factor mulspace body:expr {
    return { do: init, body };
  }
  / init:term (_ ";")+ _ body:expr {
  	return { do: init, body };
  }
  / init:term (_ ";")* { return init; }

term "term"
  = add

factor "factor"
  = affect

add
  = head:mul tail:(_ ("+" / "-") _ mul)* {
    return tail.reduce((left, e) => ({ bin: e[1], left, right: e[3] }), head);
  }

mul
  = head:affect tail:(_ ("*" / "/") _ affect)* {
    return tail.reduce((left, e) => ({ bin: e[1], left, right: e[3] }), head);
  }

affect
  = head:call tail:(_ "!")* {
    return tail.reduce((affect, _) => ({ affect }), head);
  }

call
  = head:effect tail:(minispaces args)* {
    return tail.reduce((call, e) => ({ call, args: e[1] }), head);
  }

args "argument list"
  = "(" _ seq:seq _ ")" { return seq; }
  / "(" _ ")" { return []; }

seq
  = head:factor mulspace tail:seq {
    return [head, ...tail];
  }
  / head:term (_ ",") _ tail:seq {
    return [head, ...tail];
  }
  / head:term (_ ",")? { return [head]; }

effect
  = ref:("io" / "list") _ "{" _ body:expr _ "}" {
    return { effect: { ref }, body };
  }
  / "(" _ expr:expr _ ")" { return expr; }
  / "(" _ ")" { return { err: "Missing content of parenthesis" } }
  / ref
  / literal

literal
  = int / simpleStr / rawStr

pattern "pattern"
  = ref:ident {
    return { ref };
  }

ref "variable"
  = ref:ident {
    return { ref };
  }

ident "identifier"
  = $ ([a-zA-Z_] [0-9a-zA-Z_]*)

int "integer"
  = ("0" / [1-9] [0-9]*) {
    return { literal: parseInt(text(), 10) };
  }

rawStr "raw string"
  = 'r#"' literal:($ ((! '"#') .)*) '"#' {
    return { literal };
  }

simpleStr "string"
  = '"' literal:($ [^"]*) '"' {
    return { literal };
  }

_ "whitespace"
  = spaces (comment spaces)*

shebang "shebang"
  = "#!" ($ [^\r\n]*) spaces

comment "comment"
  = "//" ($ [^\r\n]*) ([\r\n]+ / ! .)

mulspace "multiline whitespace"
  = [ \t]* ([\n\r] / comment) spaces

spaces "blank"
  = [ \t\n\r]*

minispaces "blank"
  = [ \t]*
`;

const resolveRef = (env: Env, ref: string): Value => {
  const value = env.bindings.get(ref);
  if (value !== undefined) {
    return value;
  }
  if (env.parent === undefined) {
    throw new Error(`Undefined ${ref}`);
  }
  return resolveRef(env.parent, ref);
};

const bind = (env: Env, ref: string, value: Value): Env => {
  const bindings = new Map<Ident, Value>([...env.bindings]);
  bindings.set(ref, value);
  return { parent: env.parent, bindings };
};

const jsnow: IOAction = {
  io() {
    return { value: Date() };
  },
};

const jslog = ({ value: content }: Value): IOAction => ({
  io() {
    // tslint:disable-next-line:no-console
    console.log(content);
    return { value: 'consoleLogResult' };
  },
});

const exhaust = (x: never): never => x;

const evaluateBinOp = (op: string, left: Value, right: Value) => {
  const k = () => {
    const l = left.value as any;
    const r = right.value as any;
    if (op === '+') {
      return (l + r);
    } else if (op === '*') {
      return (l * r);
    } else {
      throw new Error(`Unknown binary operator ${op}`);
    }
  };
  return { value: k() } as Value;
};

interface EvalContext {
  env: Env;
  effect: string;
  context: {};
}

const evaluate = (expr: Expr, context: EvalContext, cont: (value: Value) => Value): Value => {
  if ('literal' in expr) {
    return cont({ value: expr.literal });
  } else if ('ref' in expr) {
    return cont(resolveRef(context.env, expr.ref));
  } else if ('binary' in expr) {
    return evaluate(expr.left, context, left =>
      evaluate(expr.right, context, right =>
        cont(evaluateBinOp(expr.binary, left, right)),
      ));
  } else if ('call' in expr) {
    if ('ref' in expr.call && expr.call.ref === 'jslog' && expr.args.length === 1) {
      return evaluate(expr.args[0], context, arg => cont({ value: jslog(arg) }));
    }
    throw new Error('not impl');
  } else if ('let' in expr) {
    return evaluate(expr.be, context, content => {
      const nextEnv = bind(context.env, expr.let.var, content);
      return evaluate(expr.in, { ...context, env: nextEnv }, cont);
    });
  } else if ('affect' in expr) {
    return evaluate(expr.affect, context, ({ value: action }) => {
      if (context.effect === 'io' && typeof action === 'object' && 'io' in action) {
        return cont(action.io());
      }
      if (context.effect === 'list' && typeof action === 'object' && action instanceof Array && context.context) {
        for (const a of action) {
          cont(a);
        }
        return { value: action }; // ignored
      }
      throw new Error('Invalid action');
    });
  } else if ('effect' in expr) {
    if ('ref' in expr.effect && expr.effect.ref === 'io') {
      return cont({
        value: {
          io: () => evaluate(expr.body, { ...context, effect: 'io', context: {} }, v => v),
        },
      });
    }
    if ('ref' in expr.effect && expr.effect.ref === 'list') {
      return cont({
        value: {
          io: () => {
            const list: Value[] = [];
            evaluate(expr.body, { ...context, effect: 'list', context: { list } }, value => {
              list.push(value);
              return value;
            });
            return { value: list };
          },
        },
      });
    }
    throw new Error(`Expected io or list: ${expr.effect}`);
  } else {
    return exhaust(expr);
  }
};

const logSampleSource = `
io {
  let now = jsnow!
  let message = "It's " + now;
  jslog(message)!
}
`;
const logSample: Expr = {
  effect: {
    ref: 'io',
  },
  body: {
    let: { var: 'now' },
    be: {
      affect: { ref: 'jsnow' },
    },
    in: {
      let: { var: 'message' },
      be: {
        binary: '+',
        left: { literal: 'It\'s ' },
        right: { ref: 'now' },
      },
      in: {
        affect: {
          call: { ref: 'jslog' },
          args: [{ ref: 'message' }],
        },
      },
    },
  },
};

const listSampleSource = `
list {
  let x = xs!
  let y = ys!
  x + y
}
`;
const listSample: Expr = {
  effect: {
    ref: 'list',
  },
  body: {
    let: { var: 'x' },
    be: { affect: { ref: 'xs' } },
    in: {
      let: { var: 'y' },
      be: { affect: { ref: 'ys' } },
      in: {
        binary: '+',
        left: { ref: 'x' },
        right: { ref: 'y' },
      },
    },
  },
};

const execute = (expr: Expr, bindings: Map<Ident, Value>): Value => {
  const defaultEnv: Env = { parent: undefined, bindings };

  const { value: action } = evaluate(expr, { env: defaultEnv, effect: 'total', context: {} }, value => value);
  if (typeof action === 'object' && 'io' in action) {
    const result = action.io();
    // tslint:disable-next-line:no-console
    console.info(result);
    return result;
  } else {
    throw new Error(`main must be an action`);
  }
};

const executeLogSample = () => {
  execute(logSample, new Map<Ident, Value>([
    ['jsnow', { value: jsnow }],
  ]));
};

const executeListSample = () => {
  const range = (start: number, end: number): Value => {
    const list: Value[] = [];
    for (let i = start; i < end; i++) {
      list.push({ value: i });
    }
    return { value: list };
  };
  execute(listSample, new Map<Ident, Value>([
    ['xs', range(1, 4)],
    ['ys', { value: [{ value: 10 }, { value: 20 }] }],
  ]));
};

export const main = () => {
  executeLogSample();
  executeListSample();
};

type DescribeFunction = (name: string, fn: () => void) => void;
type ItFunction = (name: string, fn: () => void) => void;
interface BddHelper {
  describe: DescribeFunction;
  it: ItFunction;
}

export const testSuite = () => {
  describe('parser', () => {
    const parser = pegGenerate(`
      expr = _ expr:add _ {
        return expr;
      }

      add = head:mul tail:(_ "+" _ mul)* {
        return tail.reduce((l, e) => ({ op: "+", l, r: e[3] }), head);
      }

      mul = head:int tail:(_ "*" _ int)* {
        return tail.reduce((l, e) => ({ op: "*", l, r: e[3] }), head);
      }

      int = digits:($ [0-9]+) {
        return parseInt(digits, 10);
      }

      _ "whitespace" = [ \t]*
    `);
    const result = parser.parse('2 + 3 * 7');
    expect(result).toEqual({ l: 2, op: '+', r: { l: 3, op: '*', r: 7 } });
  });

  describe('evaluateBinOp', () => {
    it('add', () => {
      expect(
        evaluateBinOp('+', { value: 1 }, { value: 2 }).value,
      ).toEqual(3);
    });
  });
};
