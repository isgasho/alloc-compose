(function() {var implementors = {};
implementors["alloc_compose"] = [{"text":"impl&lt;Primary, Fallback&gt; Freeze for <a class=\"struct\" href=\"alloc_compose/struct.FallbackAlloc.html\" title=\"struct alloc_compose::FallbackAlloc\">FallbackAlloc</a>&lt;Primary, Fallback&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fallback: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;Primary: Freeze,&nbsp;</span>","synthetic":true,"types":["alloc_compose::fallback_alloc::FallbackAlloc"]},{"text":"impl Freeze for <a class=\"struct\" href=\"alloc_compose/struct.NullAlloc.html\" title=\"struct alloc_compose::NullAlloc\">NullAlloc</a>","synthetic":true,"types":["alloc_compose::null_alloc::NullAlloc"]},{"text":"impl&lt;Small, Large&gt; Freeze for <a class=\"struct\" href=\"alloc_compose/struct.SegregateAlloc.html\" title=\"struct alloc_compose::SegregateAlloc\">SegregateAlloc</a>&lt;Small, Large&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Large: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;Small: Freeze,&nbsp;</span>","synthetic":true,"types":["alloc_compose::segregate_alloc::SegregateAlloc"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()