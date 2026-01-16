<?php

function without_indent(): void
{
    $markup = <<<HTML
        <div class="post--{$post['post_type']}">
            <a href="/l/{$post['uri']}">
                {$f($post['id'])}
            </a>

            <span>
                {$f(
                    $post['post_status'] && $other_condition
                        ? esc_html($very_log_variable_name)
                        : rewrite_content_output(get_post_content()),
                )}
            </span>
        </li>
        HTML;
}

function with_indent(): void
{
    $markup = <<<HTML
            <div class="post--{$post['post_type']}">
                <a href="/l/{$post['uri']}">
                    {$f($post['id'])}
                </a>

                <span>
                    {$f(
                        $post['post_status'] && $other_condition
                            ? esc_html($very_log_variable_name)
                            : rewrite_content_output(get_post_content()),
                    )}
                </span>
            </li>
        HTML;
}

function quote(): void
{
    $markup = "
        <div class=\"post--{$post['post_type']}\">
        <a href=\"/l/{$post['uri']}\">
                {$f($post['id'])}
            </a>

            <span>
                {$f(
                    $post['post_status'] && $other_condition
                        ? esc_html($very_log_variable_name)
                        : rewrite_content_output(get_post_content()),
                )}
            </span>
        </li>
    ";
}
