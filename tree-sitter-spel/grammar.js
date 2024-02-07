/**
 * @file SPML grammar for tree-sitter
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

		_object_item: $ => seq(
			choice(
				prec(2, $.global_function),
				prec(1, $.object),
				prec(1, $.interpolated_anchor),
				prec(1, $.number),
				prec(1, $.boolean),
				$.string,
			),
			repeat(
				choice(
					$.array_offset,
					seq(
						'.',
						choice(
							prec(2, $.method),
							prec(1, $.field),
							$.interpolated_string,
							$.interpolated_anchor,
						),
					),
				),
			),
		),
		_expression_item: $ => prec(23, choice(
			$.bracketed_expression,
			$.number,
			$.expression,
			$.unary_expression,
			$.ternary_expression,
		)),
		_condition_item: $ => choice(
			$.bracketed_condition,
			$.boolean,
			$.condition,
			$.unary_condition,
			$.expression_comparison,
			$.equality_comparison,
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
		field: $ => $._word,
		method: $ => $._function,
		array_offset: $ => seq(
			choice(
				$.interpolated_string,
				$.object,
				$.global_function,
			),
			'[',
			$._expression_item,
			']',
		),
		_function: $ => seq(
			$._word,
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
		argument: $ => choice(
			prec(3, $._expression_item),
			prec(2, $._condition_item),
			prec(1, $._object_item),
			$.interpolated_string,
		),
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
					prec(1, $._condition_item),
					prec(1, $._expression_item),
				),
				$.equality_comparison_operator,
				choice(
					prec(2, $.interpolated_string),
					prec(1, $._object_item),
					prec(1, $._condition_item),
					prec(1, $._expression_item),
				),
			),
		),
		equality_comparison_operator: $ => choice(
			prec(PREC.EQUALITY, '=='),
			prec(PREC.EQUALITY, '!='),
		),

		_string_item: $ => repeat1(
			choice(
				$._string_content,
				$.escaped_string,
				$.interpolated_string,
				$.interpolated_anchor,
			),
		),
		_string_content: $ => prec(-1, /[^"$!]+/),

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
			repeat(
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
