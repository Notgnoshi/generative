from pyparsing import (
    CaselessKeyword,
    Literal,
    OneOrMore,
    Optional,
    ParserElement,
    Word,
    alphas,
    delimitedList,
    infixNotation,
    oneOf,
    opAssoc,
    pyparsing_common,
)

ParserElement.enablePackrat()

COLON = Literal(":")
LESS_THAN = Literal("<")
GREATER_THAN = Literal(">")
ARROW = Literal("->")
LEFT_PAREN = Literal("(")
RIGHT_PAREN = Literal(")")

PITCH_UP = Literal("^")
PITCH_DOWN = Literal("v")
# TODO: Does the dual uses of '<' and '>' pose any parsing problems?
ROLL_CCW = Literal("<")
ROLL_CW = Literal(">")
YAW_LEFT = Literal("-")
YAW_RIGHT = Literal("+")
PUSH_STACK = Literal("[")
POP_STACK = Literal("]")
FLIP_DIRECTION = Literal("|")

l_variable = pyparsing_common.identifier
l_value = pyparsing_common.real

l_token_name = (
    pyparsing_common.identifier
    | PITCH_UP
    | PITCH_DOWN
    | ROLL_CCW
    | ROLL_CW
    | YAW_LEFT
    | YAW_RIGHT
    | PUSH_STACK
    | POP_STACK
    | FLIP_DIRECTION
)
l_token_parameter_list = LEFT_PAREN + delimitedList(l_variable) + RIGHT_PAREN
###################################################################################################
# Arithmetic expression parsing
###################################################################################################
l_arithmetic_operand = l_variable | l_value

# TODO: Add sqrt, nroot, pi, logs, rand
l_arithmetic_expression = infixNotation(
    l_arithmetic_operand,
    [
        (oneOf("+ -"), 1, opAssoc.RIGHT),
        (Literal("**"), 2, opAssoc.LEFT),
        (oneOf("* /"), 2, opAssoc.LEFT),
        (oneOf("+ -"), 2, opAssoc.LEFT),
    ],
)
l_comparison_op = oneOf("< > <= >= != ==")
l_comparison_expression = infixNotation(
    l_arithmetic_expression, [(l_comparison_op, 2, opAssoc.LEFT)]
)

l_token_expression_list = LEFT_PAREN + delimitedList(l_expression) + RIGHT_PAREN
l_token_lhs = l_token_name + Optional(l_token_parameter_list)
l_token_rhs = l_token_name + Optional(l_token_expression_list)

l_probability = pyparsing_common.real
l_rule_lhs = Optional(l_token_lhs + LESS_THAN) + l_token_lhs + Optional(GREATER_THAN + l_token_lhs)
l_rule_rhs = OneOrMore(l_token_rhs)

# TODO: Add AND, OR, NOT
l_boolean_expression = None
l_rule = l_rule_lhs + Optional(COLON + (l_boolean_expression | l_probability)) + ARROW + l_rule_rhs

# TODO: Constants
# TODO: Ignore
