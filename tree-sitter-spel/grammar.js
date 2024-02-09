/**
 * @file SPEL grammar for tree-sitter
 * @author Mario Schäper
 * @license MIT
 */

/* eslint-disable arrow-parens */
/* eslint-disable camelcase */
/* eslint-disable-next-line spaced-comment */
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
	// https://introcs.cs.princeton.edu/java/11precedence/
	TERNARY: 1,        // ?:
	OR: 2,             // ||
	AND: 3,            // &&
	EQUALITY: 4,       // ==  !=
	COMPARISON: 5,     // <  <=  >  >=
	ADD: 6,            // +  -
	MULTIPLY: 7,       // *  /  %
	POWER: 8,          // ^
	UNARY: 9,          // +  -  !
};

module.exports = grammar({
	name: 'spel',

	extras: $ => [
		/\s+/,
	],

	word: $ => $._word,

	conflicts: $ => [
		[$._object_item, $._expression_item],
		[$._object_item, $._condition_item],
	],

	rules: {

		document: $ => $._object_item,

		_object_item: $ => choice(
			prec(2, $.method_access),
			prec(1, $.field_access),
			prec(1, $.array_offset),
			prec(1, $.global_function),
			prec(1, $.interpolated_anchor),
			prec(1, $.number),
			prec(1, $.boolean),
			prec(1, $.string),
			$.object,
		),

		_expression_item: $ => prec.left(choice(
			$.interpolated_string,
			$.bracketed_expression,
			$.number,
			$.expression,
			$.unary_expression,
			$.ternary_expression,
		)),
		_condition_item: $ => choice(
			$.condition,
			$.bracketed_condition,
			$.unary_condition,
			$.expression_comparison,
			$.equality_comparison,
			prec(-1, $.boolean),
		),
		bracketed_expression: $ => seq(
			'(',
			$._expression_item,
			')',
		),
		bracketed_condition: $ => seq(
			'(',
			$._condition_item,
			')',
		),

		object: $ => $._word,
		string: $ => seq(
			'\'',
			repeat(
				choice(
					/[^'$!]+/,
					$.escaped_string,
					prec(1, $.interpolated_string),
					prec(2, $.interpolated_anchor),
					prec(3, '$'),
					prec(4, '!'),
				),
			),
			'\'',
		),
		escaped_string: $ => choice(
			'\\b',
			'\\t',
			'\\n',
			'\\f',
			'\\r',
			'\\"',
			'\\\'',
			'\\\\',
			/\\u[0-9a-fA-F]{4}/,
		),
		number: $ => prec(10, /[0-9]+(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/),
		boolean: $ => choice(
			'true',
			'false',
		),
		global_function: $ => $._function,
		field: $ => choice(
			prec(1, $.interpolated_string),
			$._word,
		),
		method: $ => $._function,
		field_access: $ => seq(
			$._object_item,
			'.',
			$.field,
		),
		method_access: $ => seq(
			$._object_item,
			'.',
			$.method,
		),
		array_offset: $ => seq(
			$._object_item,
			'[',
			$._expression_item,
			']',
		),
		_function: $ => seq(
			$.function_name,
			'(',
			optional(
				seq(
					$.argument,
					repeat(
						seq(
							',',
							$.argument,
						),
					),
				),
			),
			')',
		),
		function_name: $ => $._word,
		argument: $ => $._object_item,
		expression: $ => prec.left(
			prec(
				2,
				seq(
					$._expression_item,
					$.expression_operator,
					$._expression_item,
				),
			),
		),
		expression_operator: $ => choice(
			prec(PREC.ADD, '+'),
			prec(PREC.ADD, '-'),
			prec(PREC.MULTIPLY, '*'),
			prec(PREC.MULTIPLY, '/'),
			prec(PREC.POWER, '^'),
			prec(PREC.MULTIPLY, '%'),
		),
		unary_expression: $ => prec(
			PREC.UNARY,
			seq(
				$.unary_expression_operator,
				$._expression_item,
			),
		),
		unary_expression_operator: $ => choice(
			prec(PREC.UNARY, '+'),
			prec(PREC.UNARY, '-'),
		),
		ternary_expression: $ => prec(
			PREC.TERNARY,
			seq(
				$._condition_item,
				'?',
				$._expression_item,
				':',
				$._expression_item,
			),
		),
		condition: $ => prec.left(
			seq(
				$._condition_item,
				$.condition_operator,
				$._condition_item,
			),
		),
		condition_operator: $ => choice(
			prec(PREC.AND, '&&'),
			prec(PREC.OR, '||'),
		),
		unary_condition: $ => prec(
			PREC.UNARY,
			seq(
				'!',
				$._condition_item,
			),
		),
		expression_comparison: $ => prec.left(
			seq(
				$._expression_item,
				$.expression_comparison_operator,
				$._expression_item,
			),
		),
		expression_comparison_operator: $ => choice(
			prec(PREC.COMPARISON, '>'),
			prec(PREC.COMPARISON, '<'),
			prec(PREC.COMPARISON, '>='),
			prec(PREC.COMPARISON, '<='),
		),
		equality_comparison: $ => prec.left(
			seq(
				choice(
					prec(2, $.interpolated_string),
					prec(1, $._object_item),
				),
				$.equality_comparison_operator,
				choice(
					prec(2, $.interpolated_string),
					prec(1, $._object_item),
				),
			),
		),
		equality_comparison_operator: $ => choice(
			prec(PREC.EQUALITY, '=='),
			prec(PREC.EQUALITY, '!='),
		),

		interpolated_string: $ => seq(
			'${',
			choice(
				$.interpolated_string,
				$._object_item,
				$._expression_item,
				$._condition_item,
			),
			'}',
		),

		interpolated_anchor: $ => seq(
			'!{',
			repeat1(
				choice(
					/[^"$\}]+/,
					$.interpolated_string,
				),
			),
			'}',
		),

		_word: $ => /[a-zA-Z_0-9]*[a-zA-Z_][a-zA-Z_0-9]*/,
	}
});
