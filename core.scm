(let (not p) (p false true))
(let (or p q) (p true q))
(let (and p q) (p q false))
(let (cons x y) (lambda (m) (m x y)))
(let (car z) (z true))
(let (cdr z) (z false))
(let
    (fold m s z) (
        (= z nil)
        s
        (fold m (m s (car z)) (cdr z))
    )
)
(let (len z) (fold (lambda (s x) (+ 1 s)) 0 z))
(let (map m z) (
    (= z nil)
    nil
    (
      (= (cdr z) nil)
      (cons (m (car z)) nil)
      (cons (m (car z)) (map m (cdr z)))
    )
  )
)
(let (filter p z) (
    (= z nil)
    nil
    ((= (cdr z) nil)
     ((p (car z)) z nil)
      (
        (p (car z))
        (cons (car z) (filter p (cdr z)))
        (filter p (cdr z))
      )
    )
  )
)
(let (showlist z) (map print z))
(let (fact n) ((= n 0) 1 (fact (- 1 n))))
