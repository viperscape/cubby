# cubby

The use cases of cubby are probably very limited. It was created specifically to fill a niche issue where avoiding a mutex wrapped hashmap was important. Threaded Component Entity Systems and streaming socket servers both would benefit from this. Currently cubby is designed to simply provide you a key for your data, you use these keys to access and reference each value where needed. A collision of data is rather rare, though I suppose on some level possible (but probably astronomical).

==

#### benchmarks ####
benchmarks are on hold, as I've done testing and retesting and haven't seen consistent results. I'm also working on a mutex-free version which should drastically speed things up, I hope
