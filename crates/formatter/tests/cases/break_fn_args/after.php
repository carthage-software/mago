<?php

$path = $this->viewCache->getCachedViewPath(
    $view->path,
    compiledView: fn() => $this->cleanupCompiled($this->compiler->compile($view->path)),
);

$path = $this->viewCache->getCachedViewPath(
    $superLonnnnnnnngVariableNameeeeeeeeeeeee->path,
    compiledView: function () {},
);
$path = $this->viewCache->getCachedViewPath(
    $superLonnnnnnnnnnnnnnnnnnnngVariableNameeeeeeeeeeeee->path,
    compiledView: function ( /* Hehe */ ) {},
);
$path = $this->viewCache->getCachedViewPath($superLonnnnnnnngVariableNameeeeeeeeeeeee->path, compiledView: function (
    /* Hehe */
) {});

function shortFunName(): void
{
    echo 'Hello';
}

function shortFunName(
    // No parameters
): void {
    echo 'Hello';
}

function shortFunName(
    // No parameters
): void {
    echo 'Hello';
}

function shortFunName( /* No parameters */ ): void
{
    echo 'Hello';
}

function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(
    /**
     * No parameters
     */
): void {
    echo 'Hello';
}

function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(
    /* No parameters */
): void {
    echo 'Hello';
}

function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(
    /// No parameters
): void {
    echo 'Hello';
}

function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(): void
{
    echo 'Hello';
}

class Example
{
    public function before(): void
    {
        echo 'Hello';
    }

    public function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(
        /**
         * No parameters
         */
    ): void {
        echo 'Hello';
    }

    public function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(
        /* No parameters */
    ): void {
        echo 'Hello';
    }

    public function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(
        /// No parameters
    ): void {
        echo 'Hello';
    }

    public function superLoooooooooooooooooooooooooooooooooooooooooooooooooooooongFuuuuuuuuuuuuuuuctioooooooooooooonNameeeeeeeeeeee(): void
    {
        echo 'Hello';
    }
}
