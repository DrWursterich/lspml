Angaben zur Datumsformatierung. Um für die Formatierung die gewünschte Sprache zu erhalten, bestehen folgende Möglichkeiten:
- Die Angabe einer Sprache über das `locale`-Attribut dieses Tags. Dies hat aber auch Einfluss auf die in `name` angegebenen Variablen.
- Übername des Locals des aktiven Publishers. Wird das `locale`-Attribut nicht verwendet, wird das Locale des aktiven Publishers verwendet. Ist kein Publisher aktiv (`in`-Modus) oder wurde im Publisher kein Locale angegeben, wird das default-Locale des Systems verwendet (im Regelfall `de_DE`).
- Angabe eines Locale in der Formatdefinition. In der Formatdefinition kann unabhängig von allen sonst definierten Formaten nur für dieses Format ein Locale angegeben werden. Dazu muss nach der Formatdefinition, mit einem Pipe-Zeichen (`|`) getrennt, das Locale angegeben werden:
```
dd.MM.yyyy HH:mm|en
```