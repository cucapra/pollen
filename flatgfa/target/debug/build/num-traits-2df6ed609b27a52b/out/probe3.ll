; ModuleID = 'probe3.fa07a04f70fac01f-cgu.0'
source_filename = "probe3.fa07a04f70fac01f-cgu.0"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx11.0.0"

; probe3::probe
; Function Attrs: uwtable
define void @_ZN6probe35probe17h38f2d108fe2083ccE() unnamed_addr #0 {
start:
  %0 = alloca [4 x i8], align 4
  store i32 1, ptr %0, align 4
  %_0.i = load i32, ptr %0, align 4
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i32 @llvm.cttz.i32(i32, i1 immarg) #1

attributes #0 = { uwtable "frame-pointer"="non-leaf" "probe-stack"="inline-asm" "target-cpu"="apple-m1" }
attributes #1 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 8, !"PIC Level", i32 2}
!1 = !{!"rustc version 1.81.0 (eeb90cda1 2024-09-04)"}
