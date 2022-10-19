# Pipes

A sequence of expressions can be written as a `pipe`, in which each stage passes it's result (`$`) to the next one: `players |? < $.health 50 | heal $`

If a stage returns something iterable, that iterator will be evaluated
and it's items be passed thru the pipe, instead of the iterator itself.
