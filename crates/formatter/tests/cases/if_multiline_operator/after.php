<?php

if (!$this->windows && !$this->getSession()
    ->getDriver() instanceof BrowserKitDriver) {
    if (!$this->getSession()->isStarted()) {
        $this->getSession()->start();
    }

    $this->windows = $this->getSession()->getWindowNames();
}

if (!$this->windows && !$this
    ->getSession()
    ->getDriver()
    ->exampleCall()
    ->gimmeMore()
    ->andAnother() instanceof BrowserKitDriver) {
    if (!$this->getSession()->isStarted()) {
        $this->getSession()->start();
    }

    $this->windows = $this->getSession()->getWindowNames();
}
