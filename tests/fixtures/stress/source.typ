#let accent = rgb("#4a90e2")
#let badge(body) = [
  #box(fill: accent)[#body]
]

= Report

#badge[Status]

```typ
= Fake Raw Heading
```

// = Fake Comment Heading

== Section

Text with #badge[inline] content.
