<?php

namespace {
    use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;

    /**
     * Creates an array.
     * @link https://php.net/manual/en/function.array.php
     * @param mixed ...$_ [optional] <p>
     * Syntax "index => values", separated by commas, define index and values.
     * index may be of type string or integer. When index is omitted, an integer index is automatically generated,
     * starting at 0. If index is an integer, next generated index will be the biggest integer index + 1.
     * Note that when two identical index are defined, the last overwrite the first.
     * </p>
     * <p>
     * Having a trailing comma after the last defined array entry, while unusual, is a valid syntax.
     * </p>
     * @return array an array of the parameters. The parameters can be given an index with the => operator.
     */
    function array(...$_) {}

    /**
     * Assigns a list of variables in one operation.
     * @link https://php.net/manual/en/function.list.php
     * @param mixed $var1 <p>A variable.</p>
     * @param mixed ...$_ [optional] <p>Another variable ...</p>
     * @return array the assigned array.
     */
    function list($var1, ...$_) {}

    /**
     * <p>Terminates execution of the script. Shutdown functions and object destructors will always be executed even if exit is called.</p>
     * <p>die is a language construct and it can be called without parentheses if no status is passed.</p>
     * @link https://php.net/manual/en/function.die.php
     * @param int|string $status [optional] <p>
     * If status is a string, this function prints the status just before exiting.
     * </p>
     * <p>
     * If status is an integer, that value will be used as the exit status and not printed. Exit statuses should be in the range 0 to 254,
     * the exit status 255 is reserved by PHP and shall not be used. The status 0 is used to terminate the program successfully.
     * </p>
     * <p>
     * Note: PHP >= 4.2.0 does NOT print the status if it is an integer.
     * </p>
     * @return void
     */
    function die($status = "") {}

    /**
     * <p>Terminates execution of the script. Shutdown functions and object destructors will always be executed even if exit is called.</p>
     * <p>exit is a language construct and it can be called without parentheses if no status is passed.</p>
     * @link https://php.net/manual/en/function.exit.php
     * @param int|string $status [optional] <p>
     * If status is a string, this function prints the status just before exiting.
     * </p>
     * <p>
     * If status is an integer, that value will be used as the exit status and not printed. Exit statuses should be in the range 0 to 254,
     * the exit status 255 is reserved by PHP and shall not be used. The status 0 is used to terminate the program successfully.
     * </p>
     * <p>
     * Note: PHP >= 4.2.0 does NOT print the status if it is an integer.
     * </p>
     * @return void
     */
    function exit($status = "") {}

    /**
     * Determine whether a variable is considered to be empty. A variable is considered empty if it does not exist or if its value
     * equals <b>FALSE</b>. <b>empty()</b> does not generate a warning if the variable does not exist.
     * @link https://php.net/manual/en/function.empty.php
     * @param mixed $var <p>Variable to be checked.</p>
     * <p>Note: Prior to PHP 5.5, <b>empty()</b> only supports variables; anything else will result in a parse error. In other words,
     * the following will not work: <b>empty(trim($name))</b>. Instead, use <b>trim($name) == false</b>.
     * </p>
     * <p>
     * No warning is generated if the variable does not exist. That means <b>empty()</b> is essentially the concise equivalent
     * to <b>!isset($var) || $var == false</b>.
     * </p>
     * @return bool <p><b>FALSE</b> if var exists and has a non-empty, non-zero value. Otherwise returns <b>TRUE</b>.<p>
     * <p>
     * The following things are considered to be empty:
     * <ul>
     * <li>"" (an empty string)</li>
     * <li>0 (0 as an integer)</li>
     * <li>0.0 (0 as a float)</li>
     * <li>"0" (0 as a string)</li>
     * <li><b>NULL</b></li>
     * <li><b>FALSE</b></li>
     * <li>array() (an empty array)</li>
     * <li>$var; (a variable declared, but without a value)</li>
     * </ul>
     * </p>
     */
    function empty($var) {}

    /**
     * <p>Determine if a variable is set and is not <b>NULL</b>.</p>
     * <p>If a variable has been unset with unset(), it will no longer be set. <b>isset()</b> will return <b>FALSE</b> if testing a variable
     * that has been set to <b>NULL</b>. Also note that a null character ("\0") is not equivalent to the PHP <b>NULL</b> constant.</p>
     * <p>If multiple parameters are supplied then <b>isset()</b> will return <b>TRUE</b> only if all of the parameters are set.
     * Evaluation goes from left to right and stops as soon as an unset variable is encountered.</p>
     * @link https://php.net/manual/en/function.isset.php
     * @param mixed $var <p>The variable to be checked.</p>
     * @param mixed ...$_ [optional] <p>Another variable ...</p>
     * @return bool Returns <b>TRUE</b> if var exists and has value other than <b>NULL</b>, <b>FALSE</b> otherwise.
     */
    function isset($var, ...$_) {}

    /**
     * <p>Destroys the specified variables.</p>
     * <p>The behavior of <b>unset()</b> inside of a function can vary depending on what type of variable you are attempting to destroy.</p>
     * @link https://php.net/manual/en/function.unset.php
     * @param mixed $var <p>The variable to be unset.</p>
     * @param mixed ...$_ [optional] <p>Another variable ...</p>
     * @return void
     */
    function unset($var, ...$_) {}

    /**
     * <p>Evaluates the given code as PHP.</p>
     * <p>Caution: The <b>eval()</b> language construct is very dangerous because it allows execution of arbitrary PHP code. Its use thus is
     * discouraged. If you have carefully verified that there is no other option than to use this construct, pay special attention not to
     * pass any user provided data into it without properly validating it beforehand.</p>
     * @link https://php.net/manual/en/function.eval.php
     * @param string $code <p>
     * Valid PHP code to be evaluated.
     * </p>
     * <p>
     * The code must not be wrapped in opening and closing PHP tags, i.e. 'echo "Hi!";' must be passed instead of '<?php echo "Hi!"; ?>'.
     * It is still possible to leave and re-enter PHP mode though using the appropriate PHP tags, e.g.
     * 'echo "In PHP mode!"; ?>In HTML mode!<?php echo "Back in PHP mode!";'.
     * </p>
     * <p>
     * Apart from that the passed code must be valid PHP. This includes that all statements must be properly terminated using a semicolon.
     * 'echo "Hi!"' for example will cause a parse error, whereas 'echo "Hi!";' will work.
     * </p>
     * <p>
     * A return statement will immediately terminate the evaluation of the code.
     * </p>
     * <p>
     * The code will be executed in the scope of the code calling <b>eval()</b>. Thus any variables defined or changed in the <b>eval()</b>
     * call will remain visible after it terminates.
     * </p>
     * @return mixed <b>NULL</b> unless return is called in the evaluated code, in which case the value passed to return is returned.
     * As of PHP 7, if there is a parse error in the evaluated code, <b>eval()</b> throws a ParseError exception. Before PHP 7, in this
     * case <b>eval()</b> returned <b>FALSE</b> and execution of the following code continued normally. It is not possible to catch a parse
     * error in <b>eval()</b> using set_error_handler().
     */
    function eval($code) {}

    /**
     * Generator objects are returned from generators, cannot be instantiated via new.
     * @link https://secure.php.net/manual/en/class.generator.php
     * @link https://wiki.php.net/rfc/generators
     *
     * @template-covariant TKey
     * @template-covariant TYield
     * @template TSend
     * @template-covariant TReturn
     *
     * @template-implements Iterator<TKey, TYield>
     */
    final class Generator implements Iterator
    {
        /**
         * Throws an exception if the generator is currently after the first yield.
         * @return void
         */
        public function rewind(): void {}

        /**
         * Returns false if the generator has been closed, true otherwise.
         * @return bool
         */
        public function valid(): bool {}

        /**
         * Returns whatever was passed to yield or null if nothing was passed or the generator is already closed.
         * @return TYield
         */
        public function current(): mixed {}

        /**
         * Returns the yielded key or, if none was specified, an auto-incrementing key or null if the generator is already closed.
         * @return TKey
         */
        #[LanguageLevelTypeAware(['8.0' => 'mixed'], default: 'string|float|int|bool|null')]
        public function key() {}

        /**
         * Resumes the generator (unless the generator is already closed).
         * @return void
         */
        public function next(): void {}

        /**
         * Sets the return value of the yield expression and resumes the generator (unless the generator is already closed).
         * @param TSend $value
         * @return TYield|null
         */
        public function send(mixed $value): mixed {}

        /**
         * Throws an exception at the current suspension point in the generator.
         * @param Throwable $exception
         * @return TYield
         */
        public function throw(Throwable $exception): mixed {}

        /**
         * Returns whatever was passed to return or null if nothing.
         * Throws an exception if the generator is still valid.
         * @link https://wiki.php.net/rfc/generator-return-expressions
         * @return TReturn
         * @since 7.0
         */
        public function getReturn(): mixed {}

        /**
         * Serialize callback
         * Throws an exception as generators can't be serialized.
         * @link https://php.net/manual/en/generator.wakeup.php
         * @return void
         */
        public function __wakeup() {}

        /**
         * @since 8.4
         */
        public function __debugInfo(): array {}
    }

    class ClosedGeneratorException extends Exception {}
}
