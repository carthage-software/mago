<?php

enum Status: int
{
    case Pending  = 0;
    case Approved = 1;
    case Rejected = 2;
}

enum HttpStatus: int
{
    case Ok                  = 200;
    case NotFound            = 404;
    case InternalServerError = 500;
}
