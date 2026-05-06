  <?php

  enum Meridiem
  {
      case AnteMeridiem;
      case PostMeridiem;
  }

  class Time
  {
      /**
       * @return int<0, 23>
       */
      public function getHours(): int
      {
          return 14;
      }

      /**
       * @return array{int<1, 12>, Meridiem}
       */
      public function getTwelveHours(): array
      {
          $hours = $this->getHours();
          $twelve_hours = $hours % 12;
          if (0 === $twelve_hours) {
              $twelve_hours = 12;
          }

          return [$twelve_hours, $hours < 12 ? Meridiem::AnteMeridiem : Meridiem::PostMeridiem];
      }
  }

  /**
   * @param int $a
   *
   * @psalm-assert-if-true int<34, 256> $a
   */
  function x(int $a): bool
  {
      return x($a);
  }

  /** @param int<min, 33>|int<257, max> $a */
  function foo(int $a): void
  {
      echo "The value of a is: $a\n";
  }

  /** @param int<34, 256> $b */
  function bar(int $b): void
  {
      echo "The value of b is: $b\n";
  }

  function baz(int $c): void
  {
      if (x($c)) {
          bar($c);
      } else {
          foo($c);
      }
  }
