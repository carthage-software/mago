<?php

/**
 * @file
 * This file contains all the valid notations for the drupal coding standard.
 *
 * The goal is to create a style checker that validates all of this
 * constructs.
 *
 * Theme files often have lists in their file block:
 * - item 1
 * - item 2
 * - sublist:
 *   - sub item 1
 *   - sub item2
 *
 * We don't want to check for class/file name matches in this test.
 * phpcs:disable Drupal.Classes.ClassFileName
 */

declare(strict_types=1);

use Drupal\mymodule\TestReturnType;
use Drupal\some_module\ExampleClass as AliasedExampleClass;
use Drupal\very_long_module_name_i_am_inventing_here_trololololo\SuperManager;

// Singleline comment before a code line.
$foo = 'bar';

/**
 * Doxygen comment style is allowed before define() statements.
 */
define('FOO_BAR', 5);

// Global variable names.
global $argc, $argv, $user, $is_https, $_mymodule_myvar;

/*
 * Multiline comment
 *
 * @see my_function()
 */

// PHP Constants must be written in CAPITAL letters.
TRUE;
FALSE;
NULL;

// Has whitespace at the end of the line.
$whitespaces = 'Yes, Please';

// Operators - have a space before and after.
$i = 0;
$i += 0;
$i -= 0;
$i == 0;
$i != 0;
$i > 0;
$i < 0;
$i >= 0;
$i <= 0;

// Unary operators must not have a space.
$i--;
--$i;
$i++;
++$i;
$x = $success ? $context['results']['success']++ : $context['results']['error']++;
$i = -1;
['i' => -1];
$i = 1 == -1;
$i = 1 === -1;
range(-50, -45);
$i[0] + 1;
$x->{$i} + 1;
REQUEST_TIME + 42;
!$x;
!($x + $y);
[-1, -2, -3];
[-1, -2, -3];

// Operator line break for long lines.
$x = 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1;

$x = $test ? -1 : 1;
$x = $test ? 1 : -1;

// Operators on new lines are allowed.
$x = CRM_Foo_Bar::singleton()->checkWhizBang($option1, $option2)
  ? $something . $notes['baz']
  : $notes['quux'] . $something_else;

// Casting has a space.
(int) $i;

// The last item in an multiline array should be followed by a comma.
// But not in a inline array.
$a = [];
$a = ['1', '2', '3'];
$a = [
  '1',
  '2',
  '3',
];
$a = ['1', '2', ['3']];
$a = [
  '1',
  '2',
  [
    'one',
    'two',
    'three',
    [
      'key' => $value,
      'title' => 'test',
    ],
  ],
];
// Short array syntax.
$a = [];
$a = ['1', '2', '3'];
$a = [
  '1',
  '2',
  '3',
];
$a = ['1', '2', ['3']];
$a = [
  '1',
  '2',
  [
    'one',
    'two',
    'three',
    [
      'key' => $value,
      'title' => 'test',
    ],
  ],
];

// Array indentation.
$x = [
  'foo' => 'bar',
  'fi' => long_function_call('hsdfsdmfsldkfnmdflkngdfngfg', 'fghfghfghfghfgh', $z),
  'a' => 'b',
  'foo' => [
    'blu' => 1,
    'f' => x(1)
      + [
        'h' => 'x',
      ],
  ],
];
$x = [
  'foo' => 'bar',
  'fi' => long_function_call('hsdfsdmfsldkfnmdflkngdfngfg', 'fghfghfghfghfgh', $z),
  'a' => 'b',
  'foo' => [
    'blu' => 1,
    'f' => x(1)
      + [
        'h' => 'x',
      ],
  ],
];

// Arrays in function calls.
foo(
  [
    'value' => 0,
    'description' => t('xyz @url', [
      '@url' => 'http://example.com',
    ]),
  ],
);
foo(
  [
    'value' => 0,
    'description' => t('xyz @url', [
      '@url' => 'http://example.com',
    ]),
  ],
);

// Pretty array layout.
$a = [
  'title' => 1,
  'weight' => 2,
  'callback' => 3,
];
$a = [
  'title' => 1,
  'weight' => 2,
  'callback' => 3,
];

// Arrays with multi line strings.
$query = db_query("
  SELECT * FROM {foobar} WHERE nid IN (1, 2, 3)
  AND date BETWEEN '%s' AND '%s'
  ", [
  ':from_date' => $from_date,
  ':to_date' => $to_date,
]);
$query = db_query("
  SELECT * FROM {foobar} WHERE nid IN (1, 2, 3)
  AND date BETWEEN '%s' AND '%s'
  ", [
  ':from_date' => $from_date,
  ':to_date' => $to_date,
]);

// Array with multi line comments.
$query = db_query(
  "
  SELECT * FROM {foobar} WHERE nid IN (1, 2, 3)
  AND date BETWEEN '%s' AND '%s'",
  /* comment
   * in here */ [
    ':from_date' => $from_date,
    ':to_date' => $to_date,
  ],
);
$query = db_query(
  "
  SELECT * FROM {foobar} WHERE nid IN (1, 2, 3)
  AND date BETWEEN '%s' AND '%s'",
  /* comment
   * in here */ [
    ':from_date' => $from_date,
    ':to_date' => $to_date,
  ],
);

// Array with multi-line constant string in it.
$array = [
  'name' => 'example_a',
  'title' => 'Example A',
  'xml' => '
<foo>
  <bar>
    123456789 123456789 123456789 123456789 123456789 123456789 123456789
    123456789 123456789 123456789 123456789 123456789 123456789 123456789
  </bar>
</foo>',
];
$array = [
  'name' => 'example_a',
  'title' => 'Example A',
  'xml' => '
<foo>
  <bar>
    123456789 123456789 123456789 123456789 123456789 123456789 123456789
    123456789 123456789 123456789 123456789 123456789 123456789 123456789
  </bar>
</foo>',
];

// Indentation: multi line function call with array and fuction closer on the
// same line.
$result = example_fetch_data($id, [
  'include_detail' => TRUE,
  'quiet' => TRUE,
]);
some_function();
$result = example_fetch_data($id, [
  'include_detail' => TRUE,
  'quiet' => TRUE,
]);
some_function();

// Indentation: multi line function call with array and closing brace on the
// same line.
watchdog(
  'example',
  'Some warning %code for %id',
  [
    '%code' => $code,
    '%doi' => $id,
  ],
  WATCHDOG_WARNING,
);
watchdog(
  'example',
  'Some warning %code for %id',
  [
    '%code' => $code,
    '%doi' => $id,
  ],
  WATCHDOG_WARNING,
);

// Nested short array syntax.
$x = [
  'label' => x(['test' => 'bar']),
];

// Nested arrays with object operators.
$derivatives["entity:$entity_type_id"] =
  [
    'label' => t('Create @entity_type path alias', ['@entity_type' => $entity_type->getLowercaseLabel()]),
    'category' => t('Path'),
    'entity_type_id' => $entity_type_id,
    'context' => [
      'entity' => ContextDefinition::create("entity:$entity_type_id")
        ->setLabel($entity_type->getLabel())
        ->setRequired(TRUE)
        ->setDescription(t('The @entity_type for which to create a path alias.', ['@entity_type' =>
          $entity_type->getLowercaseLabel()])),
      'alias' => ContextDefinition::create('string')
        ->setLabel(t('Path alias'))
        ->setRequired(TRUE)
        ->setDescription(t(
          "Specify an alternative path by which the content can be accessed. For example, 'about' for an about page. Use a relative path and do not add a trailing slash.",
        )),
    ],
    'provides' => [],
  ] + $base_plugin_definition;

$derivatives["entity:$entity_type_id"] =
  [
    'label' => t('Create @entity_type path alias', ['@entity_type' => $entity_type->getLowercaseLabel()]),
    'category' => t('Path'),
    'entity_type_id' => $entity_type_id,
    'context' => [
      'entity' => ContextDefinition::create("entity:$entity_type_id")
        ->setLabel($entity_type->getLabel())
        ->setRequired(TRUE)
        ->setDescription(t('The @entity_type for which to create a path alias.', ['@entity_type' =>
          $entity_type->getLowercaseLabel()])),
      'alias' => ContextDefinition::create('string')
        ->setLabel(t('Path alias'))
        ->setRequired(TRUE)
        ->setDescription(t(
          "Specify an alternative path by which the content can be accessed. For example, 'about' for an about page. Use a relative path and do not add a trailing slash.",
        )),
    ],
    'provides' => [],
  ] + $base_plugin_definition;

$test = [
  'columns' => $columns,
  'indexes' => [],
  'foreign keys' => [
    'format' => [
      'table' => 'filter_format',
      'columns' => ['format' => 'format'],
    ],
    'file_managed' => [
      'table' => 'file_managed',
      'columns' => ['fid' => 'carousel_image'],
    ],
  ],
];

// Arrays by reference in arrays.
$x = ['foo'];
$y = [&$x];

$x = ['foo'];
$y = [&$x];

// Multi-line function call with array and anonymous function.
multiline_call(Inspector::assertAllCallable([
  'strchr',
  [$x, 'callMe'],
  ['test', 'callMeStatic'],
  function () {
    return TRUE;
  },
]));
multiline_call(Inspector::assertAllCallable([
  'strchr',
  [$x, 'callMe'],
  ['test', 'callMeStatic'],
  function () {
    return TRUE;
  },
]));

// Nested array indentation with closures.
$options = [
  'value' => [
    'Callback' => [
      'callback' => function ($value, ExecutionContextInterface $context) {
        TheaterItem::theaterValidate($value, $context);
      },
    ],
  ],
];

$options = [
  'value' => [
    'Callback' => [
      'callback' => function ($value, ExecutionContextInterface $context) {
        TheaterItem::theaterValidate($value, $context);
      },
    ],
  ],
];

$test = [
  'original' => '$no_index_value_scalar = TRUE;',
  'settings' => [
    'no_index_value_foo' => [
      'foo' => [
        'value' => (object) [
          'value' => NULL,
          'required' => TRUE,
          'comment' => 'comment',
        ],
      ],
    ],
  ],
  'expected' => <<<'EXPECTED'
    $no_index_value_scalar = TRUE;
    $no_index_value_foo['foo']['value'] = NULL; // comment
    EXPECTED,
];

$test = [
  'original' => '$no_index_value_scalar = TRUE;',
  'settings' => [
    'no_index_value_foo' => [
      'foo' => [
        'value' => (object) [
          'value' => NULL,
          'required' => TRUE,
          'comment' => 'comment',
        ],
      ],
    ],
  ],
  'expected' => <<<EXPECTED
    abc
    def
    EXPECTED,
];

// Item assignment operators must be prefixed and followed by a space.
$a = ['one' => '1', 'two' => '2'];
foreach ($a as $key => $value) {
}

// If conditions have a space before and after the condition parenthesis.
if (TRUE || TRUE) {
  $i;
}
elseif (TRUE && TRUE) {
  $i;
}
// Commenting is allowed here.
else {
  $i;
}

while (1 == 1) {
  if (TRUE || TRUE) {
    $i;
  }
  else {
    $i;
  }
}

switch ($condition) {
  case 1:
    $i;
    break;

  case 2:
    $i;
    break;

  // Blank line after the case statement is allowed.
  case 3:
    $i;
    break;

  default:
    $i;
}

// Empty case statements are allowed.
switch ($code) {
  case 200:
  case 304:
    break;

  case 301:
  case 302:
  case 307:
    call_function();
    break;

  default:
    $result->error = $status_message;
}

/**
 * Allowed switch() statement in a function without breaks.
 */
function foo() {
  switch ($x) {
    case 1:
      return 5;

    case 2:
      return 6;

    case 3:
      return [
        'whiz',
        'bang',
      ];

    case 4:
      return helper_func(
        'whiz',
        'bang',
      );

    default:
      throw new Exception();
  }
}

do {
  $i;
} while ($condition);

/**
 * Short description.
 *
 * We use doxygen style comments.
 * What's sad because eclipse PDT and
 * PEAR CodeSniffer base on phpDoc comment style.
 * Makes working with drupal not easier :|
 *
 * @param string $field1
 *   Doxygen style comments.
 * @param int $field2
 *   Doxygen style comments.
 * @param bool $field3
 *   Doxygen style comments.
 * @param bool $field4
 *   Don't check for & in docblock of referenced variable.
 *
 * @return array
 *   Doxygen style comments.
 *
 * @see example_reference()
 * @see Example::exampleMethod()
 * @see https://www.drupal.org
 * @see http://example.com/see/documentation/is/allowed/to/exceed/eighty/characters
 */
function foo_bar($field1, $field2, $field3 = NULL, &$field4 = NULL) {
  $system['description'] = t('This module inserts funny text into posts randomly.');
  return $system[$field];
}

// Function call.
$var = foo($i, $i, $i);

// Multiline function call.
$var = foo(
  $i,
  $i,
  $i,
);

// Multiline function call with array.
$var = foo(
  [
    $i,
    $i,
    $i,
  ],
  $i,
  $i,
);

// Multiline function call with only one array.
$var = foo(
  [
    $i,
    $i,
    $i,
  ],
);

/**
 * Enter description here ...
 */
function _private_foo() {
}

// When calling class constructors with no arguments, always include
// parentheses.
$bar = new Bar();

$bar = new Bar($arg1, $arg2);

$bar = 'Bar';
$foo = new $bar();
$foo = new $bar($i, $i);

// Static class variables use camelCase.
Bar::$basePath = '/foo';

// Concatenation - there has to be a space.
$i . 'test';
$i . 'test';
$i . $i;
$i . NULL;

/**
 * Nice allignment in functions.
 */
function test_test2() {
  $a = 5;
  $aa = 6;
  $aaa = 7;
}

/**
 * Example of chained method invocations in a function.
 */
function test3() {
  // Uses the Rules API as example.
  $rule = rule();
  $rule
    ->condition('rules_test_condition_true')
    ->condition('rules_test_condition_true')
    ->condition(
      rules_or()
        // Inline test comment that continues on a
        // second line.
        ->condition(rules_condition('rules_test_condition_true')->negate())
        ->condition('rules_test_condition_false')
        ->condition(
          rules_and()
            ->condition('rules_test_condition_false')
            ->condition('rules_test_condition_true')
            ->negate(),
        ),
    );
}

// Test usages of t().
t('special character: \"');
t("special character: \'");
// Escaping is allowed here because we make use of the other quote type, too.
t('Link to Drupal\'s <a href="@url">admin pages</a>.', ['@url' => url('admin')]);

// Test inline comment style.
// Comment one.
t('x');
// Comment two.
// @todo this is valid!
t('x');
// Goes on?
t('x');
/* Longer comment
 * bla bla
 * end!
 */
t('x');
// @see http://example.com/with/a/very/long/link/that/is/longer/than/80/characters
t('x');
// http://example.com/with/a/very/long/link/that/is/longer/than/80/characters/really
t('x');
// @see my_function()
t('x');
// Some text here, then a reference.
// @see \Drupal\rules\Entity\ReactionRuleStorage
t('x');
// t() refers to a function name and should not be capitalized.
t('x');
// rules_admin is a fancy machine name word with underscores and should not be
// capitalized and ignored.
t('x');
// {} this comment start should not be flagged.
t('x');
// Some code examples in inline comments:
// @code
//   function mymodule_menu() {
//     $items['abc/def'] = array(
//       'page callback' => 'mymodule_abc_view',
//     );
//     return $items;
//   }
// @endcode
// @todo A long description what is left to be done. Exceeds the first line and
//   is indented on the second line.
// Now follows a list:
// - item 1
//   additional description line.
// - item 2
// - item 3 comes with a sub list:
//   - sub item 1
//   - sub item 2
//     description of sub item 2.
// List is closed, normal indentation here. Now comes a paragraph empty line.
//
// And here continues the long comment.
// Now some UTF-8 characters that do not exceed 80 characters.
// Hà Nội là thủ đô, đồng thời là thành phố đứng đầu Việt Nam về diện tích tự.
//
// Further information:
// - @link https://api.drupal.org/api/drupal/groups/8 All topics @endlink
// - @link https://www.drupal.org/project/examples Examples project (sample modules) @endlink
// - @link https://www.drupal.org/list-changes API change notices @endlink
// - @link https://www.drupal.org/developing/api/8 Drupal 8 API longer references @endlink
t('x');

// People want to pretty-format their numbered lists, do not throw an error for
// this:
// 1. A link has the current path in its 'data-drupal-link-system-path'
//    attribute.
// 2. We are on the front page and a link has the special '<front>' value in
//    its 'data-drupal-link-system-path' attribute.
t('x');

// Concatenating some symbols is fine.
$x = '(' . t('Test') . ')';
$x = '[' . t('Test') . ']';
$x = '- ' . t('Test') . ' -';
$x = '<' . t('Test') . '>';
$x = '«' . t('Test') . '»';
$x = "\n" . t('Test') . "\n";

/**
 * Doc block with some code tags.
 *
 * Some description and here comes the code:
 * @code
 *   $options = drupal_parse_url($_GET['destination']);
 *   $my_url = url($options['path'], $options);
 *   $my_link = l('Example link', $options['path'], $options);
 * @endcode
 *
 * Some more description here. And now the parameters.
 *
 * @param int $x
 *   Parameter comment here.
 */
function test1($x) {
}

/**
 * Variable amount of parameters is allowed with the "..." notation.
 *
 * Example:
 * @code
 *   test_invoke_all('node_view', $node, $view_mode);
 * @endcode
 *
 * @param string $hook
 *   The name of the hook to invoke.
 * @param ...
 *   Arguments to pass to the hook.
 */
function test_invoke_all($hook) {
}

// Test string concatenation.
$x =
  'This string is very long and thus it can and should be concatenated.'
  . 'Othverwise the string will be very hard to maintain and or read';
$x = 'This string should be concatenated. Even if it is just a little bit ' . 'longer';
$x =
  (1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)
  . 'This is initially '
  . 'short but since it has a lot of code before the real text it can be '
  . 'concatenated';

// Requiring files conditionally is allowed.
if ($a == TRUE) {
  require_once 'good.tpl.php';
}

/**
 * This is not an implementation of hook_foobar().
 */
function hook_foobar() {
}

/**
 * Implements hook_foo().
 *
 * @link http://example.com/long-link-here-whatever Some long documentation link @endlink
 */
function mymodule_foo() {
}

/**
 * Implements hook_foo_BAR_ID_bar() for some_type_bar().
 */
function mymodule_foo_some_type_bar() {
}

/**
 * Implements hook_foo_bar() for foo_bar.tpl.php.
 */
function mymodule_foo_bar_phptemplate() {
}

/**
 * Implements hook_foo_bar() for foo-bar.html.twig.
 */
function mymodule_foo_bar_twig() {
}

/**
 * Implements drush_hook_foo_bar().
 */
function drush_mymodule_foo_bar() {
}

/**
 * Implements hook_foo_bar() for blocks.
 */
function mymodule_foo_bar_block() {
}

/**
 * Not documenting all parameters is allowed.
 *
 * @param \Drupal\node\Entity\Node $node
 *   The loaded node entity that we will use to do whatever.
 */
function mymodule_form_callback($form, &$form_state, Node $node) {
}

$x = 'Some markup text<br />';

// Inline if statements with ?: are allowed.
$x = $y == $z ? 23 : 42;

// Standard watchdog message.
watchdog('mymodule', 'Log message here.');

// For assigning by reference it is allowed that there is no space after the
// "=" oeprator.
$batch = &batch_get();

// Security issue: https://www.drupal.org/node/750148
preg_match('/.+/i', 'subject');
preg_match('/.+/imsuxADSUXJ', 'subject');
preg_filter('/.+/i', 'replacement', 'subject');
preg_replace('/.+/i', 'replacement', 'subject');
// Use a not so common delimiter.
preg_match('@.+@i', 'subject');
preg_match('@.+@imsuxADSUXJ', 'subject');
preg_filter('@.+@i', 'replacement', 'subject');
preg_replace('@.+@i', 'replacement', 'subject');
preg_match("/test(\d+)/is", 'subject');

// Correctly formed try/catch block.
try {
  do_something();
}
catch (Exception $e) {
  scream();
}

$result = $x ?: FALSE;

t('Some long mulit-line
  text is weird, but allowed.');

// Anonymous functions should not throw indentation errors here.
$test = array_walk($fragments, function (&$item) {
  if (strpos($item, '%') === 0) {
    $item = '%';
  }
});

/**
 * Doc tags are allowed to exceed 80 characters.
 *
 * @param \Drupal\very_long_module_name_i_am_inventing_here_trololololo\SuperManager $x
 *   The super duper manager comment goes here.
 * @param \Traversable $y
 *   Some PHP core class/interface.
 */
function test2(SuperManager $x, \Traversable $y) {
}

/**
 * Test comment.
 *
 * @throws \Drupal\locale\StringStorageException
 *   The exception description is here and should not cause an error.
 */
function test4() {
}

/**
 * Test chained method indentation which should not throw errors.
 */
function test5() {
  $mock
    ->expects($this->any())
    ->method('findTranslation')
    ->will($this->returnCallback(function ($argument) use ($translations) {
      if (isset($translations[$argument['language']][$argument['source']])) {
        return (object) ['translation' => $translations[$argument['language']][$argument['source']]];
      }
      return TRUE;
    }));
}

/**
 * Array syntax with brackets is allowed in type hints.
 *
 * @param string[] $names
 *   An indexed array of names.
 */
function test6(array $names) {
}

[, , $bundle] = entity_extract_ids('node', $entity);

l("<i class='icon icon-industrial-building'></i>", 'node/add/job', [
  'attributes' => ['title' => t('add job')],
  'html' => TRUE,
]);

/**
 * Some short description.
 *
 * @param string $x
 *   Some parameter.
 *
 * @todo These are allowed here.
 */
function test7($x) {
}

$x = 'Some markup text with allowed HTML5 <br> tag';

/**
 * Some comment with exclamation mark!
 *
 * And the long description!
 *
 * @param int $x
 *   Exclamation mark allowed!
 *
 * @throws \MyException
 *   Exclamation mark allowed!
 */
function test8($x) {
}

/**
 * Make sure that link tags are allowed in long descriptions.
 *
 * For forcing it to a boolean TRUE or FALSE please use
 * @link MenuItem::forceAccessCallbackTRUE @endlink() or
 * @link MenuItem::forceAccessCallbackFALSE @endlink() as you see fit.
 */
function test9() {
}

/**
 * Link tags in the long description before param tags are allowed.
 *
 * Visit also:
 * @link https://www.drupal.org/node/323101 Strings at well-known places: built-in menus, .. @endlink.
 *
 * @section more_info Further information
 *
 * - @link https://api.drupal.org/api/drupal/groups/8 All topics @endlink
 * - @link https://www.drupal.org/project/examples Examples project (sample modules) @endlink
 * - @link https://www.drupal.org/list-changes API change notices @endlink
 * - @link https://www.drupal.org/developing/api/8 Drupal 8 API longer references @endlink
 *
 * @param string $title
 *   The untranslated title of the menu item.
 */
function test10($title) {
}

/**
 * Some description.
 *
 * @todo Here is a very long link that exceeds 80 charaters on the next line:
 *   http://example.com/test/long/link/with/stuff/here/making/it/even/longer/now/so/that/it/shows
 */
function test11() {
}

/**
 * Parameter docs with a long nested list.
 *
 * @param string $a
 *   Lists are usually preceded by a line ending in a colon:
 *   - Item in the list.
 *   - Another item.
 *     - key: Sub-list with keys, first item.
 *     - key2: (optional) Second item with a key.
 *   - Back to the outer list. Sometimes list items are quite long, in which
 *     case you can wrap the text like this.
 *   - Last item in the outer list.
 *   Text that is outside of the list continues here.
 */
function test13($a) {
}

/**
 * Using an alias type hint but the fully qualified name in the docs.
 *
 * @param \Drupal\some_module\ExampleClass $a
 *   Example parameter.
 */
function test14(AliasedExampleClass $a) {
}

// camelCase and snake_case variables are allowed.
$snake_case = 1;
$camelCase = 1;

// It should be possible to use a parenthesis in a comment without having the
// InlineCommentSniff complain. (About the the last character of the line.)
$comment = 'fine';

/**
 * Comments ending with parentheses.
 *
 * It should be possible to use a parenthesis in a comment without having the
 * DocCommentSniff complain. (About the the last character of the line.)
 *
 * @param string $foo
 *   A parameter comment is also allowed to end (with parenthesis)
 */
function test15($foo) {
}

/**
 * Variable arguments with type declared.
 *
 * Variable arguments with type declared should not result in an error
 * when no named argument is given in the param definition.
 *
 * @param string ...
 *   Variable number of string arguments.
 */
function test16() {
}

$access =
  $account->hasPermission('translate configuration')
  && $mapper->hasSchema()
  && $mapper->hasTranslatable()
  && $source_language_access;

/**
 * Return with optional null and function with some void returns.
 *
 * When a function has an optional null as a return value then it should not
 * result in an error when there is a void return in the function.
 *
 * @param string $arg
 *   An argument.
 *
 * @return int|null
 *   Return has an optional null.
 */
function test17($arg) {
  if ($arg === 1) {
    return;
  }
  return 1;
}

/**
 * Indentation of multi-line function declarations is standardized.
 */
function test18(
  ConfigFactoryInterface $config_factory,
  EntityTypeManagerInterface $entity_type_manager,
  CacheTagsInvalidatorInterface $cache_invalidator,
  ModuleHandlerInterface $module_handler,
  EntityFieldManagerInterface $entity_field_manager,
  EntityTypeBundleInfoInterface $entity_type_bundle_info,
) {
  return 0;
}

/**
 * Object operator indentation is fine like this.
 */
function is_table_empty($table_name, $database) {
  return (
    !$database->schema()->tableExists($table_name)
    || !$database
      ->select($table_name)
      ->countQuery()
      ->range(0, 1)
      ->execute()
      ->fetchField()
  );
}

/**
 * Another example of correct object operator indentation.
 */
function test19($key, $value, $connection) {
  $connection
    ->merge('foo')
    ->keys([
      'name' => $key,
      'collection' => 'collection',
    ])
    ->fields(['value' => $value])
    ->execute();
}

// Correct object operator indentation.
ContentLanguageSettings::create([
  'target_entity_type_id' => 'entity_test',
  'target_bundle' => 'some_bundle',
])
  ->setLanguageAlterable(TRUE)
  ->setDefaultLangcode('authors_default')
  ->save();

/**
 * Another example of correct operator indentation.
 */
function test20($link, $parents, $connection) {
  return $connection
    ->insert('book')
    ->fields([
      'nid' => $link['nid'],
      'bid' => $link['bid'],
      'pid' => $link['pid'],
      'weight' => $link['weight'],
    ] + $parents)
    ->execute();
}

/**
 * PHP 7 type hints are allowed.
 *
 * @param string $a
 *   Parameter one.
 * @param int $b
 *   Parameter two.
 */
function test21(string $a, int $b) {
}

/**
 * Implements hook_very_long_module_name_neverending_hook_name_that_is_a_long_cat_alter().
 */
function mymodule_very_long_module_name_neverending_hook_name_that_is_a_long_cat_alter() {
}

/**
 * Numbers in types are allowed.
 *
 * @param \Drupal\x123_module\MyInterface $a
 *   Parameter type with numbers in it.
 */
function test22(MyInterface $a) {
}

/**
 * Using PHP 7 return types is allowed.
 *
 * @return \Drupal\mymodule\TestReturnType
 *   What we return.
 */
function test23(): TestReturnType {
  return foo();
}

/**
 * Variadic arguments with proper declaration.
 *
 * @param \Drupal\mymodule\Element ...$element
 *   The variadic parameter comment.
 */
function test24(Element ...$element) {
}

// Allow longer than 80 character array definitions on one line (up to 120).
if (!in_array($form_object->getOperation(), ['add', 'edit', 'delete', 'clone', 'default'], TRUE)) {
  return;
}
if ($content_type_facet_field && in_array($entity_type, ['entity:node', 'entity:media'])) {
  $foo->setContentTypeFacetValues($item, $entity_type, $content_type_facet_field);
}

// Allow array definition statements over 80 but under 120 characters.
$form['strings'] = [
  '#type' => 'table',
  '#tree' => TRUE,
  '#language' => $langname,
  '#header' => [
    $this->t('Source string'),
    $this->t('Translation for @language on this longer line', ['@language' => $langname]),
    $this->t('Delete'),
  ],
  '#empty' => $this->t('No strings available.'),
  '#attributes' => ['class' => ['locale-translate-edit-table']],
];

/**
 * Doc block is here and an ignore directive is ok.
 */
// phpcs:ignore Drupal.NamingConventions.ValidClassName
function phpcs_ignore_comment() {
}

/**
 * Test for @phpstan-ignore-next-line.
 *
 * Coder issue https://www.drupal.org/project/coder/issues/3516489
 */
// @phpstan-ignore-next-line missingType.return
function ignore_phpstan_comment() {
}

/**
 * Test for @phpstan-ignore with attribute before.
 */
#[ExampleAttribute('foo', 'bar')]
// @phpstan-ignore-next-line missingType.return
function ignore_phpstan_comment_with_attribute_before() {
}

/**
 * Test that nested array types are ok.
 *
 * @param array<array<scalar|null>|object|scalar|null> $param
 *   A complex nested array type.
 *
 * @return array<array<scalar|null>|object|scalar|null>
 *   An array of results.
 */
function pdo_weird_return_type($param) {
  return pdo();
}

/**
 * Comments are allowed to end in 3 dots...
 */
function comment_test_dots() {
}

$x = new class() {

  /**
   * This is a test comment.
   *
   * @var array
   */
  public private(set) array $data = [];

};

// Test mago compatibility with single array value over multiple lines.
$response = new AjaxResponse();
$response->addCommand(new InvokeCommand('#edit-field-job-expiration-date-0-value-date', 'val', [$end_value->format(
  'Y-m-d',
)]));

foreach ($result['subscriptions'] as $subscription) {
  $group = $subscription->organization->entity;
  $context->addCacheableDependency($group);
  $context->addCacheableDependency($subscription);
  $context->addCacheTags([
    'node:' . \Drupal::service('example_organization.organization_helper')->getGroupOrganizationProfile($group)->id(),
  ]);
}

$form['frontend'] = [
  '#type' => 'select',
  '#required' => FALSE,
  '#title' => $this->t('Frontend'),
  '#description_display' => $this->t('Select the frontend'),
  '#options' =>
    ['all' => $this->t('All menus'), 'default' => $this->t('Default menus')] + example_common_get_available_frontends(),
  '#default_value' => $this->getRequest()->query->get('frontend') ?? 'all',
  '#name' => 'selected-frontend',
];
$form['frontend'] = [
  '#options' =>
    ['all' => $this->t('All menus'), 'default' => $this->t('Default menus')]
      + example_common_get_available_frontends()
      + example2_common_get_available_frontends(),
];

\Drupal::database()
  ->insert(DataManager::DATA_TABLE)
  ->fields(['uid', 'last_sent', 'reminders'], [
    $user->id(),
    \Drupal::time()->getCurrentTime()
      - (($i + 1) * (int) \Drupal::config('example_auth.settings')->get('email_verification_timeout')),
    $i,
  ])
  ->execute();

static::setMessage(t('The combined export file could not be created: @message.', ['@message' =>
  $e->getMessage()]), 'error');

$form['account']['notify_message'] = [
  '#type' => 'container',
  '#markup' =>
    '<div class="messages messages--warning">'
      . t(
        "By turning this off, no welcome or verification emails will be sent to the account that's being created.",
      )
      . '</div>',
  '#states' => [
    'visible' => [
      ':input[name="notify"]' => ['checked' => FALSE],
    ],
  ],
];

$query = $this->database->query('SELECT t.name, t.tid
  FROM {taxonomy_term_field_data} t
  WHERE t.vid = :vocabulary
  AND t.name IN (:termNames[])', [
  ':vocabulary' => $vocabulary,
  ':termNames[]' => $term_names,
]);
