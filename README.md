# whismur

a bot to track The Discourse, on irc

whismur started with a joke: "it has been [0] days since #rust-offtopic discussed communism". so i
decided to actually track the count, and made a bot to do it for me.

to flag whismur, start a message with its name, a comma or colon, then the topic itself. for
example:

```text
<misdreavus> [whismur]: communism
-[whismur]:#rust-offtopic- It has been [4] days since #rust-offtopic discussed "communism".
* [whismur] erases the board, writes [0]
-[whismur]:#rust-offtopic- The previous record was [15] days.
```

telling whismur about The Discourse will automatically reset its timer. there is no way to ask about
the timer without resetting it. if chat wasn't discussing it before, it probably will when you ask
how long it's been!

whismur is licensed GPL 3.0 or later. see LICENSE for details.
