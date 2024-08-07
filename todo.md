Consider before beta

* consider int as u32 instead of usize (u64). sizes?
  -> not worth it, most are used as indexes into vec or the like,
  so it's quite annoying. probably doesn't matter much anyway.

* repackage menu et al
    * use commons
