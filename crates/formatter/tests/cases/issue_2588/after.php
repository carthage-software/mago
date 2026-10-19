<?php

(clone $a)->p->push()->only();
foo((clone $a)->people->push($x)->only('uuid')->values());

(clone $a)?->p->push()->only();
(clone $a)->p?->push()?->only();
(clone $a)->push()->only();
(clone $a)?->push()?->only();

(clone $a)->people;
(clone $a)->method();
(clone $a)->p->c()->d()->e();

(clone $a->people)->p->push()->only();
(clone $a->method())->p->push()->only();
(clone ($a ?? $b))->p->push()->only();

(clone $a)->people->push($x)->only('uuid')->values();

clone $a->p->push()->only();
