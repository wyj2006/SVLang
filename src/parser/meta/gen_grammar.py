import os
import tokenize
from io import StringIO
from pprint import *
from token import *

from gen_expression import *
from meta_ast import *
from meta_parser import MetaParser
from pegen.tokenizer import Tokenizer

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

i = 0
while i < len(grammar.rules):
    if grammar.rules[i].name in [
        "primary_literal",
        "description",
        "module_nonansi_header",
        "module_ansi_header",
        "hierarchical_variable_identifier",
        "decimal_number",
        "octal_number",
        "binary_number",
        "hex_number",
        "fixed_point_number",
        "function_subroutine_call",
        "hierarchical_tf_identifier",
    ]:
        grammar.rules[i].is_silent = True
    if grammar.rules[i].name in [
        "triple_quoted_string",
        "quoted_string",
        "string_literal",
        "unbased_unsized_literal",
    ]:
        grammar.rules[i].is_atomic = True
    if grammar.rules[i].name in [
        "time_literal",
        "number",
        "integral_number",
        "real_number",
    ]:
        grammar.rules[i].is_compound_atom = True
    if grammar.rules[i].name in [
        "block_event_expression",
        "incomplete_class_scoped_type",
        "module_path_conditional_expression",
        "module_path_expression",
        "property_expr",
        "sequence_expr",
        "select_expression",
        "event_expression",
        "constant_expression",
        "identifier",
        "constant_select",
        "constant_bit_select",
        "casting_type",
        "constant_primary",
        "primary",
        "expression_or_cond_pattern",
        "expression",
        "conditional_expression",
        "method_call",
        "casting_type",
        "constant_assignment_pattern_expression",
        "cond_pattern",
        "constant_multiple_concatenation",
        "constant_function_call",
        "method_call_root",
        "inside_expression",
        "constant_concatenation",
        "constant_cast",
        "constant_let_expression",
        "unary_operator",
        "binary_operator",
        "covergroup_variable_identifier",
        "formal_identifier",
        "array_identifier",
        "conditional_statement",
        "inc_or_dec_expression",
    ]:
        grammar.rules.pop(i)
        continue
    i += 1

with open(
    os.path.join(os.path.dirname(__file__), "..", "grammar.pest"),
    mode="w",
    encoding="utf-8",
) as file:
    file.write(grammar.to_pest())
    file.write(
        open(
            os.path.join(os.path.dirname(__file__), "custom.pest"), encoding="utf-8"
        ).read()
    )
    file.write(gen_infix_grammar())
gen_infix_parser()
