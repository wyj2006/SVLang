import os
import tokenize
from io import StringIO
from pprint import *
from token import *

from meta_ast import *
from meta_parser import MetaParser
from pegen.tokenizer import Tokenizer


def eliminate_left_recursive(grammar: Grammar):
    rules: dict[str, Rule] = {}
    firsts = {}
    for rule in grammar.rules:
        firsts[rule.name] = rule.first()
        rules[rule.name] = rule

    graph = {}
    for rule_name, first in firsts.items():
        if rule_name not in graph:
            graph[rule_name] = set()
        graph[rule_name] = [i.name for i in first if isinstance(i, Name)]

    scanned = set()
    left_recursive = {}

    def dfs(path: tuple):
        if path[-1] not in graph:
            return
        for i in graph[path[-1]]:
            if i in path:
                if i not in left_recursive:
                    left_recursive[i] = set()
                left_recursive[i].add(path[path.index(i) + 1 :])
                continue
            if i in scanned:
                continue
            scanned.add(i)
            dfs(path + (i,))

    for rule_name in graph:
        if rule_name not in scanned:
            dfs((rule_name,))

    eliminated = True
    for rule_name, paths in left_recursive.items():
        paths = list(paths)
        rule = rules[rule_name]
        # 直接左递归
        if any([len(path) == 0 for path in paths]):
            recursive = []
            non_recursive = []
            match rule.item.normalize():
                case Choice(items):
                    for i in items:
                        if any(
                            [
                                j.name == rule_name
                                for j in i.first()
                                if isinstance(j, Name)
                            ]
                        ):
                            recursive.append(i)
                        else:
                            non_recursive.append(i)
                case _:
                    raise Exception(f"不存在非递归项: {rule.to_pest()}")
            if not non_recursive:
                raise Exception(f"不存在非递归项: {rule.to_pest()}")
            prime_items = []
            for i in recursive:
                match i:
                    case Sequence(items):
                        prime_items.append(Sequence(items[1:]))
                    case _:
                        raise NotImplementedError
            i = 2
            prime_name = f"{rule_name}{i}"
            while prime_name in rules:
                i += 1
                prime_name = f"{rule_name}{i}"
            prime_rule = Rule(prime_name, Choice(prime_items).normalize())
            rules[prime_name] = prime_rule
            grammar.rules.append(prime_rule)

            choice_items = []
            for i in non_recursive:
                choice_items.append(Sequence((i, Repeat(Name(prime_name)))))
            object.__setattr__(rule, "item", Choice(choice_items).normalize())
            continue
        eliminated = False
        # 间接左递归
        new_item = rule.item
        for path in paths:
            new_item = new_item.replace(Name(path[0]), rules[path[0]].item)
        object.__setattr__(rule, "item", new_item.normalize())
    return eliminated


code = (
    open(os.path.join(os.path.dirname(__file__), "grammar.bnf"), encoding="utf-8")
    .read()
    .replace("::=", "=")
)

tokeniter = []
tokengen = tokenize.generate_tokens(StringIO(code).readline)
for token in tokengen:
    if token.exact_type not in [NEWLINE, INDENT, DEDENT]:
        tokeniter.append(token)

verbose = False
tokenizer = Tokenizer(iter(tokeniter), verbose=verbose)
parser = MetaParser(tokenizer, verbose=verbose)
grammar: Grammar = parser.start()
for i in range(2**32):
    if eliminate_left_recursive(grammar):
        break
else:
    raise Exception("左递归无法在指定次数内消除")

with open(
    os.path.join(os.path.dirname(__file__), "..", "grammar.pest"),
    mode="w",
    encoding="utf-8",
) as file:
    file.write(grammar.to_pest())
    file.write(r"""
quoted_string_item        = @{ !("\"" | NEWLINE | "\\") ~ ASCII }
triple_quoted_string_item = @{ !"\\" ~ ASCII }
COMMENT                   = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" | "/" ~ (!NEWLINE ~ ANY)* ~ NEWLINE }
c_identifier              = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
escaped_identifier        = @{ "\\" ~ !WHITESPACE ~ ASCII ~ WHITESPACE }
simple_identifier         = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_" | "$")* }
system_tf_identifier      = @{ "$" ~ (ASCII_ALPHANUMERIC | "_" | "$")+ }
WHITESPACE                =  { " " | "\t" | NEWLINE }
string_escape_seq         = @{ "\\x" ~ ASCII_HEX_DIGIT{1, 2} | "\\" ~ ASCII_OCT_DIGIT{1, 3} | "\\" ~ ASCII }
file_path_spec            =  { "/" | "\\" }
""")
