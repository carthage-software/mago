<?php

clone $foo;
clone ($foo?->bar);
clone ($foo?->bar ?? $foo?->baz);
