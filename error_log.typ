#set page(
  paper: "a4",
  margin: (top: 2.2cm, bottom: 2.2cm, left: 2cm, right: 2cm),
  header: context {
    if counter(page).get().first() > 1 [
      #grid(
        columns: (1fr, auto),
        align(left)[#text(size: 8.5pt, fill: rgb("555555"))[IDM Navigator – Ereignis- & Fehlerprotokoll]],
        align(right)[#text(size: 8.5pt, fill: rgb("555555"))[CrustyPump Report]]
      )
      #v(2pt)
      #line(length: 100%, stroke: 0.5pt + rgb("cbd5e1"))
    ]
  },
  footer: context [
    #line(length: 100%, stroke: 0.5pt + rgb("cbd5e1"))
    #v(3pt)
    #grid(
      columns: (1fr, auto),
      align(left)[#text(size: 8pt, fill: rgb("718096"))[Quelle: TimescaleDB / IDM Navigator Weboberfläche]],
      align(right)[#text(size: 8pt, fill: rgb("718096"))[Seite #counter(page).display()]]
    )
  ]
)

#set text(font: "Liberation Sans", size: 9pt, lang: "de")
#set par(justify: true, leading: 0.6em)

#let data = json("error_history.json")

// Header Banner
#align(center)[
  #block(width: 100%, fill: rgb("f1f5f9"), inset: 14pt, radius: 6pt, stroke: 1pt + rgb("cbd5e1"))[
    #text(size: 17pt, weight: "bold", fill: rgb("0f172a"))[IDM Wärmepumpe – Fehler- & Ereignisprotokoll]\
    #v(3pt)
    #text(size: 10pt, fill: rgb("475569"))[Generiert am #data.last_updated | Gesamtanzahl Einträge: #data.total_count]
  ]
]

#v(10pt)

== 1. Häufigkeitsstatistik der erfassten Ereignisse

#v(4pt)

#table(
  columns: (3fr, 1fr),
  fill: (x, y) => if y == 0 { rgb("e2e8f0") } else if calc.even(y) { rgb("f8fafc") } else { none },
  stroke: 0.5pt + rgb("cbd5e1"),
  table.header([*Ereigniskategorie*], [*Anzahl*]),
  ..data.summary.map(s => (
    [#s.category],
    [#s.count]
  )).flatten()
)

#pagebreak()

== 2. Vollständige chronologische Ereignishistorie

#v(4pt)

#table(
  columns: (1.8fr, 0.9fr, 4fr),
  fill: (col, row) => {
    if row == 0 {
      rgb("1e293b")
    } else if calc.even(row) {
      rgb("f8fafc")
    } else {
      none
    }
  },
  stroke: 0.3pt + rgb("e2e8f0"),
  table.header(
    [*#text(fill: white)[Zeitstempel]*],
    [*#text(fill: white)[Code]*],
    [*#text(fill: white)[Meldung / Beschreibung]*]
  ),
  ..data.history.map(e => (
    [#text(size: 8pt)[#e.datetime]],
    if e.code in ("036", "615", "044", "101") [
      #text(weight: "bold", fill: rgb("dc2626"))[#e.code]
    ] else if e.code in ("997", "516", "620") [
      #text(weight: "bold", fill: rgb("ea580c"))[#e.code]
    ] else if e.is_ack [
      #text(fill: rgb("94a3b8"))[#e.code]
    ] else [
      #e.code
    ],
    if e.code in ("036", "615", "044", "101") [
      #text(weight: "bold", fill: rgb("b91c1c"))[#e.message]
    ] else if e.is_ack [
      #text(fill: rgb("64748b"))[#e.message]
    ] else [
      #e.message
    ]
  )).flatten()
)
