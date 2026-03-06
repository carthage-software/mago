<?php

// try to catch, same-line comment
try {
    doSomething();
} // @todo Handle this properly
catch (Exception $e) {
    handleError($e);
}

// catch to finally, same-line comment
try {
    doSomething();
} catch (Exception $e) {
    handleError($e);
} // end catch
finally {
    cleanup();
}

// try to catch to catch chain, same-line comments
try {
    doSomething();
} // first comment
catch (RuntimeException $e) {
    handleRuntime($e);
} // second comment
catch (Exception $e) {
    handleGeneral($e);
}

// try to catch, next-line comment
try {
    doSomething();
}
// This comment is on the next line
catch (Exception $e) {
    handleError($e);
}

// try to finally, no catch
try {
    doSomething();
} // cleanup needed
finally {
    cleanup();
}

// catch to finally, next-line comment
try {
    doSomething();
} catch (Exception $e) {
    handleError($e);
}
// next-line before finally
finally {
    cleanup();
}

// no comment, regression guard
try {
    doSomething();
} catch (Exception $e) {
    handleError($e);
} finally {
    cleanup();
}
