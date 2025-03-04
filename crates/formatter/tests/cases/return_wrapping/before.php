<?php

class Example {
    public function something(): void {
        return $handler->run(function (string $output, string $buffer) use ($log): bool {
            if ($output === Process::ERR) {
                return true;
            }
        
            if ($line = trim($buffer)) {
                $log($line);
            }
        
            return true;
        }) === 0;
    }

}