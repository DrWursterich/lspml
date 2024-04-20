Angaben zur Dezimalformatierung. Um für die Formatierung die gewünschte Sprache zu erhalten, bestehen folgende Möglichkeiten:
- Die Angabe einer Sprache über das `locale`-Attribut dieses Tags. Dies hat aber auch Einfluss auf die in `name` angegebenen Variablen.
- Übername des Locals des aktiven Publishers. Wird das `locale`-Attribut nicht verwendet, wird das Locale des aktiven Publishers verwendet. Ist kein Publisher aktiv (`in`-Modus) oder wurde im Publisher kein Locale angegeben, wird das default-Locale des Systems verwendet (im Regelfall `de_DE`).
- Angabe eines Locale in der Formatdefinition. In der Formatdefinition kann unabhängig von allen sonst definierten Formaten nur für dieses Format ein Locale angegeben werden. Dazu muss nach der Formatdefinition mit einem Pipe-Zeichen (`|`) getrennt, das Locale angegeben werden:
```
##.00|en
```

__Hinweis__: *Bis Version 2.0.2 wurde der Doppelpunkt als Trennzeichen verwendet. Da dateformat diese Funktion ab Version 2.0.3 auch besitzt konnte der Doppelpunkt nicht mehr verwendet werden, da dieser Teil der Format-Definition sein kann. Aus diesem Grund wurde der Doppelpunkt als Locale-Trennzeichen als deprecated deklariert.*