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

if ($node->bundle() == 'organization_profile' && \Drupal::service('example_recruiter_verification.example_recruiter_verification_helper')
    ->isRecruiterVerificationEnabled()) {
    $a = $b;
}

if ($node->bundle() == 'organization_profile' && \Drupal::service('example_recruiter_verification.example_recruiter_verification_helper')
    ->somePublicProperty) {
    $a = $b;
}

if ($node->bundle() == 'organization_profile' && $object->method('example_recruiter_verification.example_recruiter_verification_helper')
    ->somePublicProperty) {
    $a = $b;
}
