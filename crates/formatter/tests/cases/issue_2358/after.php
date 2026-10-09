<?php

class SomeTest extends TestCase
{
    public function testHandle(): void
    {
        $this->request = new Request([], [], [], [], [], ['REQUEST_URI' => '/']);
        $this->request->attributes->add(['account' => ['someaccount']]);
        $response = $this->mw->handle($this->request, function () {
        });

        $this->assertSame(400, $response->getStatusCode());
    }
}
