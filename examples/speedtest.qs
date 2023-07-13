n = num = 20000;

print("started...");
1.upto(n).each(i -> {
    s = 1.upto(num).reject(n->{0==num%i}).reduce(0, Integer::"+");
    #reduce(0, Integer::"+");


    (num == s).then(s.display);
    print(("num="+num)+(",s="+s));
    :1.num = num - 1;
});

