(define-trait xcall-receiver-trait
  (
    (handle-call-message ((string-ascii 150) (buff 1024) (list 50 (string-ascii 150))) (response bool uint))
  )
)