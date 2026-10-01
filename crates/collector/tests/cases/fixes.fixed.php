<?php

// @mago-expect analysis:used
first();

// @mago-expect analysis:used
last();

// @mago-expect analysis:used, analysis:counted(2)
middle();

// @mago-expect analysis:used, analysis:counted(2)
partial();

// @mago-expect analysis:used
shared();

// @mago-expect analyser:used
alias();

// @mago-ignore analysis:used
ignored();
