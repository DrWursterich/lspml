Decodiert die auszugebende Zeichenkette mit dem angegebenen Encoding. Es ist
möglich eine kommaseparierte Liste von Encodings anzugeben, die nacheinander
ausgeführt werden. Gültige Werte sind:
- `none` kein decoding
- `xml` decoded XML-Text:
    `&lt;` zu `<`
    `&gt;` zu `>`
    `&apos;` zu `'`
    `&quot;` zu `"`
    `&amp;` zu `&`
- `url` decoded eine URL (entsprechend dem Charset des Publishers)
- `base64` decoded eine BASE64 encodete Zeichenkette
- `escff (ab Version 2.0.3.26)` decodet die mit dem `escff`-encoding
encodierten Zeichenketten.