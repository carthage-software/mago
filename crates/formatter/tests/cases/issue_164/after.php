<?php

final class Issue164
{
    private function test(): Response
    {
        $this->assertStringEqualsStringIgnoringLineEndings(<<<HTML
            ||||||||||||||||||||||||||||||||
                                            
            ||||||||||||||||||||||||        
                                    
            ||||||||||||||||        
                            
            ||||||||        
                    
            ||||    
                
            ||  
            HTML, $html);

        $this->assertStringEqualsStringIgnoringLineEndings('
        ||||||||||||||||||||||||||||||||
                                        
        ||||||||||||||||||||||||        
                                
        ||||||||||||||||        
                        
        ||||||||        
                
        ||||    
            
        ||  
        ', $html);

        $this->assertStringEqualsStringIgnoringLineEndings('
        ||||||||||||||||||||||||||||||||
                                        
        ||||||||||||||||||||||||        
                                
        ||||||||||||||||        
                        
        ||||||||        
                
        ||||    
            
        ||  
        ', $html);

        $this->assertStringEqualsStringIgnoringLineEndings(
            "
        ||||||||||||||||||||||||||||||||
                                        
        ||||||||||||||||||||||||        
                                
        ||||||||||||||||        
                        
        ||||||||        
                
        ||||    
            
        || $tag
        ",
            $html,
        );

        $message = Str\format(<<<MESSAGE
            Shell command "%s" returned an exit code of "%d".

            STDOUT:
                %s

            STDERR:
                %s
            MESSAGE, $command, $code, Str\replace($stdout_content, PHP_EOL, PHP_EOL . '    '), Str\replace($stderr_content, PHP_EOL, PHP_EOL . '    '));
    }
}
