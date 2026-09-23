import os
import tokenize
from io import StringIO
from pprint import *
from token import *

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
    if grammar.rules[i].name in ["number", "integral_number", "primary_literal"]:
        grammar.rules[i].is_silent = True
    if grammar.rules[i].name in [
        "triple_quoted_string",
        "quoted_string",
        "string_literal",
        "unbased_unsized_literal",
        "decimal_base",
        "binary_base",
        "octal_base",
        "hex_base",
        "unsigned_number",
        "binary_value",
        "octal_value",
        "hex_value",
    ]:
        grammar.rules[i].is_atomic = True
    if grammar.rules[i].name in [
        "real_number",
        "fixed_point_number",
        "binary_number",
        "octal_number",
        "binary_number",
        "decimal_number",
        "time_literal",
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
        "cond_predicate",
        "constant_primary",
        "expression",
        "method_call",
        "casting_type",
        "cond_pattern",
        "primary",
        "expression_or_cond_pattern",
        "constant_multiple_concatenation",
        "constant_function_call",
        "conditional_expression",
        "inside_expression",
        "constant_cast",
        "constant_let_expression",
        "unary_operator",
        "binary_operator",
        "array_identifier",
        "covergroup_variable_identifier",
        "formal_identifier",
        "constant_assignment_pattern_expression",
        "constant_concatenation",
        "method_call_root",
        "identifier",
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
    file.write(r"""
keywords                  =  { "accept_on" | "alias" | "always" | "always_comb" | "always_ff" | "always_latch" | "and" | "assert" | "assign" | "assume" | "automatic" | "before" | "begin" | "bind" | "bins" | "binsof" | "bit" | "break" | "buf" | "bufif0" | "bufif1" | "byte" | "case" | "casex" | "casez" | "cell" | "chandle" | "checker" | "class" | "clocking" | "cmos" | "config" | "const" | "constraint" | "context" | "continue" | "cover" | "covergroup" | "coverpoint" | "cross" | "deassign" | "default" | "defparam" | "design" | "disable" | "dist" | "do" | "edge" | "else" | "end" | "endcase" | "endchecker" | "endclass" | "endclocking" | "endconfig" | "endfunction" | "endgenerate" | "endgroup" | "endinterface" | "endmodule" | "endpackage" | "endprimitive" | "endprogram" | "endproperty" | "endspecify" | "endsequence" | "endtable" | "endtask" | "enum" | "event" | "eventually" | "expect" | "export" | "extends" | "extern" | "final" | "first_match" | "for" | "force" | "foreach" | "forever" | "fork" | "forkjoin" | "function" | "generate" | "genvar" | "global" | "highz0" | "highz1" | "if" | "iff" | "ifnone" | "ignore_bins" | "illegal_bins" | "implements" | "implies" | "import" | "incdir" | "include" | "initial" | "inout" | "input" | "inside" | "instance" | "int" | "integer" | "interconnect" | "interface" | "intersect" | "join" | "join_any" | "join_none" | "large" | "let" | "liblist" | "library" | "local" | "localparam" | "logic" | "longint" | "macromodule" | "matches" | "medium" | "modport" | "module" | "nand" | "negedge" | "nettype" | "new" | "nexttime" | "nmos" | "nor" | "noshowcancelled" | "not" | "notif0" | "notif1" | "null" | "or" | "output" | "package" | "packed" | "parameter" | "pmos" | "posedge" | "primitive" | "priority" | "program" | "property" | "protected" | "pull0" | "pull1" | "pulldown" | "pullup" | "pulsestyle_ondetect" | "pulsestyle_onevent" | "pure" | "rand" | "randc" | "randcase" | "randsequence" | "rcmos" | "real" | "realtime" | "ref" | "reg" | "reject_on" | "release" | "repeat" | "restrict" | "return" | "rnmos" | "rpmos" | "rtran" | "rtranif0" | "rtranif1" | "s_always" | "s_eventually" | "s_nexttime" | "s_until" | "s_until_with" | "scalared" | "sequence" | "shortint" | "shortreal" | "showcancelled" | "signed" | "small" | "soft" | "solve" | "specify" | "specparam" | "static" | "string" | "strong" | "strong0" | "strong1" | "struct" | "super" | "supply0" | "supply1" | "sync_accept_on" | "sync_reject_on" | "table" | "tagged" | "task" | "this" | "throughout" | "time" | "timeprecision" | "timeunit" | "tran" | "tranif0" | "tranif1" | "tri" | "tri0" | "tri1" | "triand" | "trior" | "trireg" | "type" | "typedef" | "union" | "unique" | "unique0" | "unsigned" | "until" | "until_with" | "untyped" | "use" | "uwire" | "var" | "vectored" | "virtual" | "void" | "wait" | "wait_order" | "wand" | "weak" | "weak0" | "weak1" | "while" | "wildcard" | "wire" | "with" | "within" | "wor" | "xnor" | "xor" }
identifier                = @{ !(keywords ~ !(ASCII_ALPHANUMERIC | "_" | "$")) ~ (simple_identifier | escaped_identifier) }
quoted_string_item        = @{ !("\"" | NEWLINE | "\\") ~ ASCII }
triple_quoted_string_item = @{ !"\\" ~ ASCII }
COMMENT                   = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" | "/" ~ (!NEWLINE ~ ANY)* ~ NEWLINE }
c_identifier              = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
escaped_identifier        = @{ "\\" ~ !WHITESPACE ~ ASCII ~ WHITESPACE }
simple_identifier         = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_" | "$")* }
system_tf_identifier      = @{ "$" ~ (ASCII_ALPHANUMERIC | "_" | "$")+ }
WHITESPACE                =  _{ " " | "\t" | NEWLINE }
string_escape_seq         = @{ "\\x" ~ ASCII_HEX_DIGIT{1, 2} | "\\" ~ ASCII_OCT_DIGIT{1, 3} | "\\" ~ ASCII }
file_path_spec            =  { "/" | "\\" }

block_event_expression = { ("begin" | "end") ~ hierarchical_btf_identifier ~ ("or" ~ block_event_expression)* }

incomplete_class_scoped_type = { type_identifier ~ ("::" ~ type_identifier_or_class_type)+ }

module_path_expression                = {
    module_path_expression_recursive_base ~ module_path_expression_recursive*
}
module_path_expression_recursive_base = {
    module_path_primary
  | unary_module_path_operator ~ attribute_instance* ~ module_path_primary
}
module_path_expression_recursive      = {
    binary_module_path_operator ~ attribute_instance* ~ module_path_expression
  | "?" ~ attribute_instance* ~ module_path_expression ~ ":" ~ module_path_expression
}

property_expr           = {
    property_expr_base ~ property_expr_recursive*
}
property_expr_base      = {
    "strong" ~ "(" ~ sequence_expr ~ ")"
  | "weak" ~ "(" ~ sequence_expr ~ ")"
  | "(" ~ property_expr ~ ")"
  | "not" ~ property_expr
  | "if" ~ "(" ~ expression_or_dist ~ ")" ~ property_expr ~ ("else" ~ property_expr)?
  | "case" ~ "(" ~ expression_or_dist ~ ")" ~ property_case_item ~ property_case_item* ~ "endcase"
  | "nexttime" ~ property_expr
  | "nexttime" ~ "[" ~ constant_expression ~ "]" ~ property_expr
  | "s_nexttime" ~ property_expr
  | "s_nexttime" ~ "[" ~ constant_expression ~ "]" ~ property_expr
  | "always" ~ property_expr
  | "always" ~ "[" ~ cycle_delay_const_range_expression ~ "]" ~ property_expr
  | "s_always" ~ "[" ~ constant_range ~ "]" ~ property_expr
  | "s_eventually" ~ property_expr
  | "eventually" ~ "[" ~ constant_range ~ "]" ~ property_expr
  | "s_eventually" ~ "[" ~ cycle_delay_const_range_expression ~ "]" ~ property_expr
  | "accept_on" ~ "(" ~ expression_or_dist ~ ")" ~ property_expr
  | "reject_on" ~ "(" ~ expression_or_dist ~ ")" ~ property_expr
  | "sync_accept_on" ~ "(" ~ expression_or_dist ~ ")" ~ property_expr
  | "sync_reject_on" ~ "(" ~ expression_or_dist ~ ")" ~ property_expr
  | sequence_expr ~ "|->" ~ property_expr
  | sequence_expr ~ "|=>" ~ property_expr
  | sequence_expr ~ "#-#" ~ property_expr
  | sequence_expr ~ "#=#" ~ property_expr
  | clocking_event ~ property_expr
  | sequence_expr
  | property_instance
}
property_expr_recursive = {
    "or" ~ property_expr
  | "and" ~ property_expr
  | "until" ~ property_expr
  | "s_until" ~ property_expr
  | "until_with" ~ property_expr
  | "s_until_with" ~ property_expr
  | "implies" ~ property_expr
  | "iff" ~ property_expr
}

sequence_expr           = {
    sequence_expr_base ~ sequence_expr_recursive*
}
sequence_expr_base      = {
    "(" ~ sequence_expr ~ ("," ~ sequence_match_item)* ~ ")" ~ sequence_abbrev?
  | "first_match" ~ "(" ~ sequence_expr ~ ("," ~ sequence_match_item)* ~ ")"
  | expression_or_dist ~ "throughout" ~ sequence_expr
  | cycle_delay_range ~ sequence_expr ~ (cycle_delay_range ~ sequence_expr)*
  | expression_or_dist ~ boolean_abbrev?
  | sequence_instance ~ sequence_abbrev?
  | clocking_event ~ sequence_expr
}
sequence_expr_recursive = {
    "and" ~ sequence_expr
  | "intersect" ~ sequence_expr
  | "or" ~ sequence_expr
  | "within" ~ sequence_expr
  | cycle_delay_range ~ sequence_expr ~ (cycle_delay_range ~ sequence_expr)*
}

select_expression           = {
    select_expression_base ~ select_expression_recursive*
}
select_expression_base      = {
    select_condition
  | "!" ~ select_condition
  | "(" ~ select_expression ~ ")"
  | cross_identifier
  | cross_set_expression ~ ("matches" ~ integer_covergroup_expression)?
}
select_expression_recursive = {
    "&&" ~ select_expression
  | "||" ~ select_expression
  | "with" ~ "(" ~ with_covergroup_expression ~ ")" ~ ("matches" ~ integer_covergroup_expression)?
}

event_expression           = {
    event_expression_base ~ event_expression_recursive*
}
event_expression_base      = {
    edge_identifier? ~ expression ~ ("iff" ~ expression)?
  | sequence_instance ~ ("iff" ~ expression)?
  | "(" ~ event_expression ~ ")"
}
event_expression_recursive = {
    "or" ~ event_expression
  | "," ~ event_expression
}

cond_predicate      = _{ expression }
constant_expression = _{ expression }
constant_primary    = _{ expression }
expression          =  { prefix? ~ primary ~ postfix? ~ (infix ~ prefix? ~ primary ~ postfix?)* }

method_call  =  { implicit_class_handle ~ "." ~ method_call_body }
casting_type =  { simple_type | signing | "string" | "const" }
primary      = _{
    "(" ~ operator_assignment ~ ")"
  | tagged_union_expression
  | inc_or_dec_expression
  | primary_literal
  | (package_scope | class_qualifier)? ~ hierarchical_identifier ~ select
  | empty_unpacked_array_concatenation
  | concatenation ~ ("[" ~ range_expression ~ "]")?
  | multiple_concatenation ~ ("[" ~ range_expression ~ "]")?
  | function_subroutine_call ~ ("[" ~ range_expression ~ "]")?
  | let_expression
  | "(" ~ mintypmax_expression ~ ")"
  | assignment_pattern_expression
  | streaming_concatenation
  | sequence_method_call
  | cast
  | "this"
  | "$"
  | "null"
  | ps_parameter_identifier ~ constant_select
  | specparam_identifier ~ ("[" ~ constant_range_expression ~ "]")?
  | genvar_identifier
  | formal_port_identifier ~ constant_select
  | (package_scope | class_scope)? ~ enum_identifier
  | type_reference
  | "null"
}

prefix         = _{ reduction_xnor | reduction_nor | reduction_nand | positive | negative | not | bit_not | reduction_and | reduction_or | reduction_xor }
positive       =  { "+" ~ attribute_instance* }
negative       =  { "-" ~ attribute_instance* }
not            =  { "!" ~ attribute_instance* }
bit_not        =  { "~" ~ attribute_instance* }
reduction_and  =  { "&" ~ attribute_instance* }
reduction_nand =  { "~&" ~ attribute_instance* }
reduction_or   =  { "|" ~ attribute_instance* }
reduction_nor  =  { "~|" ~ attribute_instance* }
reduction_xor  =  { "^" ~ attribute_instance* }
reduction_xnor =  { ("~^" | "^~") ~ attribute_instance* }

infix             = _{
    case_eq
  | case_neq
  | wildcard_eq
  | wildcard_neq
  | arithmetic_rshift
  | arithmetic_lshift
  | logical_eq
  | triple_amp
  | eq
  | neq
  | and
  | or
  | pow
  | le
  | ge
  | xnor
  | rshift
  | lshift
  | implication
  | cond_then
  | cond_or
  | add
  | sub
  | mul
  | div
  | lt
  | gt
  | bit_and
  | bit_or
  | xor
}
cond_then         =  { "?" ~ attribute_instance* }
cond_or           =  { ":" }
add               =  { "+" ~ attribute_instance* }
sub               =  { "-" ~ attribute_instance* }
mul               =  { "*" ~ attribute_instance* }
div               =  { "/" ~ attribute_instance* }
eq                =  { "==" ~ attribute_instance* }
neq               =  { "!=" ~ attribute_instance* }
case_eq           =  { "===" ~ attribute_instance* }
case_neq          =  { "!==" ~ attribute_instance* }
wildcard_eq       =  { "==?" ~ attribute_instance* }
wildcard_neq      =  { "!=?" ~ attribute_instance* }
and               =  { "&&" ~ attribute_instance* }
or                =  { "||" ~ attribute_instance* }
pow               =  { "**" ~ attribute_instance* }
lt                =  { "<" ~ attribute_instance* }
le                =  { "<=" ~ attribute_instance* }
gt                =  { ">" ~ attribute_instance* }
ge                =  { ">=" ~ attribute_instance* }
bit_and           =  { "&" ~ attribute_instance* }
bit_or            =  { "|" ~ attribute_instance* }
xor               =  { "^" ~ attribute_instance* }
xnor              =  { ("^~" | "~^") ~ attribute_instance* }
rshift            =  { ">>" ~ attribute_instance* }
arithmetic_rshift =  { ">>>" ~ attribute_instance* }
lshift            =  { "<<" ~ attribute_instance* }
arithmetic_lshift =  { "<<<" ~ attribute_instance* }
implication       =  { "->" ~ attribute_instance* }
logical_eq        =  { "<->" ~ attribute_instance* }
triple_amp        =  { "&&&" }

postfix        = _{ inside | method_call_op | cond_pattern | cast_op }
inside         =  { "inside" ~ "{" ~ range_list ~ "}" }
method_call_op =  { "." ~ method_call_body }
cond_pattern   =  { "matches" ~ pattern }
cast_op        =  { "'" ~ "(" ~ expression ~ ")" }
""")
