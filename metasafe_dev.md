## Relevant Notes:
### Analysis:
# Goal:
Try using local shadows. We will compare the cost of SFI
Problem with SFI is memory allocation and shadowing every pointer seems expensive.
Currently MetaSafe computes the address of every shadow instead of getting one pointer at a time.
The challenge here will be whether we can avoid this.

# The Plan:
If a certain inhouse object is accessed for a write,
write to a shadow. From the first write in a given function scope, use the shadow, including all reads succeeding the first write. At the end of the function, if the object is still alive, then:
- Enable MPK
- Write everything from the shadow back to the object.
- Disable MPK.
The question at this point becomes whether we can enable and disable MPK safely.

# Further optimizations?
The assumption is that all such inhouse objects are on the heap. If we can confirm that such an object is stack-based, then
MPK need not be enabled. As it will reside on the safe stack anyway.
This analysis will only be done locally. We can't really afford inter-procedural PTA, as we saw how it affects compile time in TRust.


## Dev notes:
# Analysis: Relevant places:
- Rustc borrow:
    > Liveness (rustc_borrow)
    > 