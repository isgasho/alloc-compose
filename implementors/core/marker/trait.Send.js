(function() {var implementors = {};
implementors["alloc_compose"] = [{"text":"impl&lt;A, const SIZE:&nbsp;usize&gt; Send for Chunk&lt;A, SIZE&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;Primary, Secondary&gt; Send for Fallback&lt;Primary, Secondary&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Primary: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;Secondary: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for Null","synthetic":true,"types":[]},{"text":"impl&lt;A, C&gt; Send for Proxy&lt;A, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Send for Region&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Send for Counter","synthetic":true,"types":[]},{"text":"impl Send for AtomicCounter","synthetic":true,"types":[]},{"text":"impl Send for FilteredCounter","synthetic":true,"types":[]},{"text":"impl Send for FilteredAtomicCounter","synthetic":true,"types":[]},{"text":"impl Send for AllocInitFilter","synthetic":true,"types":[]},{"text":"impl Send for ReallocPlacementFilter","synthetic":true,"types":[]},{"text":"impl Send for ResultFilter","synthetic":true,"types":[]},{"text":"impl&lt;Alloc:&nbsp;Send, Prefix, Suffix&gt; Send for Affix&lt;Alloc, Prefix, Suffix&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()