<?php

// @mago-expect analysis:unused, analysis:used
first();

// @mago-expect analysis:used, analysis:unused
last();

// @mago-expect analysis:used, analysis:unused, analysis:counted(2)
middle();

// @mago-expect analysis:used, analysis:counted(3)
partial();

// @mago-expect analysis:unused, used
shared();

// @mago-expect analysis:unused, analyser:used
alias();

// @mago-ignore analysis:used, analysis:unused
ignored();
