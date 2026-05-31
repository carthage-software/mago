<?php

return new Config()
    // grouped by target state
    ->allowTransition(A::class, B::class)
    ->allowTransition(C::class, D::class);
