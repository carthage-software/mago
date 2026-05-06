  <?php

  /** @param Closure $callback */
  function configureScope(Closure $callback): mixed
  {
      return $callback(1, 2, 3);
  }

  configureScope(function (string $x): string {
      return 'A' . $x;
  });

  /** @param callable $callback */
  function configureScopeWithCallable(callable $callback): mixed
  {
      return $callback(1, 2, 3);
  }

  configureScopeWithCallable(function (string $x): string {
      return 'A' . $x;
  });
