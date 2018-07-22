(module
 (import "env" "memory" (memory $0 1))
 (table 0 anyfunc)
 (export "main" (func $main))
 (export "malloc" (func $malloc))
 (export "free" (func $free))
 (export "realloc" (func $realloc))
 (export "memalign" (func $memalign))
 (export "stackSave" (func $stackSave))
 (export "stackAlloc" (func $stackAlloc))
 (export "stackRestore" (func $stackRestore))
 (func $main (; 0 ;) (result i32)
  (i32.const 0)
 )
 (func $malloc (; 1 ;) (param $0 i32) (result i32)
  (i32.const 0)
 )
 (func $free (; 2 ;) (param $0 i32)
 )
 (func $realloc (; 3 ;) (param $0 i32) (param $1 i32) (result i32)
  (i32.const 0)
 )
 (func $memalign (; 4 ;) (param $0 i32) (param $1 i32) (result i32)
  (i32.const 0)
 )
 (func $not_a_malloc (; 5 ;) (param $0 i32) (param $1 i32) (result i32)
  (i32.const 0)
 )
 (func $stackSave (; 6 ;) (result i32)
  (i32.load offset=4
   (i32.const 0)
  )
 )
 (func $stackAlloc (; 7 ;) (param $0 i32) (result i32)
  (local $1 i32)
  (set_local $1
   (i32.load offset=4
    (i32.const 0)
   )
  )
  (i32.store offset=4
   (i32.const 0)
   (i32.and
    (i32.sub
     (get_local $1)
     (get_local $0)
    )
    (i32.const -16)
   )
  )
  (get_local $1)
 )
 (func $stackRestore (; 8 ;) (param $0 i32)
  (i32.store offset=4
   (i32.const 0)
   (get_local $0)
  )
 )
)
;; METADATA: { "asmConsts": {},"staticBump": 12, "initializers": [] }
