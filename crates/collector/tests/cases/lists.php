<?php

/** @mago-expect analysis:first(1),second(2),third(3) Expected issues, with counts. */
foo();

/** @mago-expect analysis:first(1),analysis:second(2),analysis:third(3) Expected issues, with counts. */
foo();

/** @mago-expect analysis:first(1), second(2), third(3) Expected issues, with counts. */
foo();

/** @mago-expect analysis:first(1), analysis:second(2), analysis:third(3) Expected issues, with counts. */
foo();

// @mago-expect analysis:first(1) , analyzer:second(2) , analyser:third(3) Expected issues, with counts.
foo();

// @mago-expect analysis:first(1),	second(2),	analysis:third(3) Expected issues, with counts.
foo();

/* @mago-ignore analysis:first(1), analysis:second(2), third(3) Expected issues, with counts. */
foo();
