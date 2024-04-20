Encodiert die auszugebende Zeichenkette mit dem angegebenen Encoding. Es ist möglich eine kommaseparierte Liste von Encodings anzugeben, die nacheinander ausgeführt werden. Gültige Werte sind:
- `none` kein encoding
- `html` encoded HTML-Text
    `<` zu `&lt;`
    `>` zu `&gt;`
    `'` zu `&#039;`
    `"` zu `&#034;`
    `&` zu `&amp;`
    wird z.B. verwendet um value-Attribute in Formularen zu füllen
- `xml` encoded XML-Text
    `<` zu `&lt;`
    `>` zu `&gt;`
    `'` zu `&apos;`
    `"` zu `&quot;`
    `&` zu `&amp;`
    und alle Zeichen außerhalb des 7-Bit ASCII-Zeichensatzes
- `script` encoded für JavaScript, JSP, o.ä (escaped `\n`, `\r`, `"` und `'`)
    `\` zu `\\` *(Ab Version 2.0.3)*
    `'` zu `\'`
    `"` zu `\"`
    `\n` zu `\\n`
    `\r` zu `\\r`
- `php` *(ab Version 2.1.0.44)* encoded für PHP (escaped `\n`, `\r`, `$`, `"` und `'`)
    `\` zu `\\`
    `'` zu `\'`
    `"` zu `\"`
    `$` zu `\$`
    `\n` zu `\\n`
    `\r` zu `\\r`
- `php;[KEY=VALUE;KEY=VALUE;...]` *(ab Version 2.12.22)* Derzeit wird nur der KEY `'ignore'` aktzeptiert, um zu definieren, welche Werte NICHT encodiert werden sollen! Mögliche Werte für den `KEY` '`ignore'` sind:
    - `backslash`
    - `singleQuote`
    - `doubleQuote`
    - `carriageReturn`
    - `newLine`
    - `backspace`
    - `tab`
    - `dollar`
    Beispiel:
    ```
    php;ignore=singleQuote;ignore=newLine
    ```
- `url` encoded eine URL (entsprechend dem Charset des Publishers)
- `url; charset=latin1` encoded eine URL (mit dem übergebenen Charset)
- `entity` encoded alle Entitäten (jedes Zeichen wird zu seinem Entitäts-Pendant)
    z.B.
    `A` zu `&#65;`
    `[SPACE]` zu `&#32;`
- `plain` encoded `<`, `>` und Zeilenenden (`\n`, `\r`, `\r\n`)
    `<` zu `&lt;`
    `>` zu `&gt;`
    `\n` zu `<br>` oder `<br/>\n`
    `\r\n` zu `<br>` oder `<br/>\r\n`
- `ascii` encoded Windows-Sonderzeichen nach ASCII
- `path` encoded einen Verzeichnisnamen
- `filename` encoded einen Dateinamen
- `wikitext` *(ab Version 2.0.3)* Erzeugt ein Wiki-Text Syntax HTML. Weitere Informationen über Wiki-Text finden sie [hier](http://de.wikipedia.org/wiki/Hilfe:Textgestaltung)
    __Deprecated (ab Version 2.1.0)__ *wikitext ist kein encoding, sondern eine Konvertierung und sollte jetzt über das Attribut convert und dem Wert wiki2html verwendet werden*
- `base64 (ab Version 2.0.1)` encoded nach BASE64
- `base64NotChunked` (ab Version 2.8)* encoded nach BASE64, fügt aber keine Zeilenumbrüche hinzu
- `hex` *(ab Version 2.0.1)* encoded nach HEX. Hierbei wird jedes Zeichen in eine Zahl umgewandelt und dessen Hex-Wert ausgegeben
- `escff` *(ab Version 2.0.3.26)* encodet alle Zeichen mit einem Byte-Wert kleiner als 128 in einen Hex-Wert, beginnend mit einem Doppelpunkt (`:`). Dieses Encoding wird dazu verwendet, von `sp:form` erzeugte Formularfelder zu encoden, wenn das Formular an eine PHP-Seite gesendet wird. Dieses Encoding ist kein Standardencoding, sondern eine proprietäre Entwicklung von Sitepark.