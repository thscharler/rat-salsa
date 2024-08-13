Consider before beta

* clone state with new focusflag

* panics in textarea
    * ? Result based API?
* sharded range-map? skiplist style? other?
* render sub range map
* replace textrange with char-ranges internally?
*

* consider int as u32 instead of usize (u64). sizes?
  -> not worth it, most are used as indexes into vec or the like,
  so it's quite annoying. probably doesn't matter much anyway.

* repackage menu et al
    * use commons
