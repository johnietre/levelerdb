// leveldb/util/histogram.{h,cc}

#![allow(dead_code)]
pub(crate) struct Histogram {
    min: f64,
    max: f64,
    num: f64,
    sum: f64,
    sum_squares: f64,

    buckets: [f64; NUM_BUCKETS],
}

impl Histogram {
    pub(crate) fn new() -> Self {
        // TODO: Subject to change?
        Self {
            min: 0.0,
            max: 0.0,
            num: 0.0,
            sum: 0.0,
            sum_squares: 0.0,
            buckets: [0.0; NUM_BUCKETS],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.min = BUCKET_LIMIT[NUM_BUCKETS - 1];
        self.max = 0.0;
        self.num = 0.0;
        self.sum_squares = 0.0;
        for i in 0..NUM_BUCKETS {
            self.buckets[i] = 0.0;
        }
    }

    pub(crate) fn add(&mut self, value: f64) {
        // NOTE: Figure out comment below
        // Linear search is fast enough for our usage in db_bench
        let mut b = 0;
        while b < NUM_BUCKETS - 1 && BUCKET_LIMIT[b] <= value {
            b += 1;
        }
        self.buckets[b] += 1.0;
        if self.min > value {
            self.min = value;
        }
        if self.max < value {
            self.max = value;
        }
        self.num += 1.0;
        self.sum += value;
        self.sum_squares += value * value;
    }

    pub(crate) fn merge(&mut self, other: &Histogram) {
        if other.min < self.min {
            self.min = other.min
        }
        if other.max > self.max {
            self.max = other.max
        }
        self.num += other.num;
        self.sum += other.sum;
        self.sum_squares += self.sum_squares;
        for b in 0..NUM_BUCKETS {
            self.buckets[b] += other.buckets[b];
        }
    }

    fn median(&self) -> f64 {
        self.percentile(50.0)
    }

    fn percentile(&self, p: f64) -> f64 {
        let threshold = self.num * (p / 100.0);
        let mut sum = 0.0;
        for b in 0..NUM_BUCKETS {
            sum += self.buckets[b];
            if sum >= threshold {
                // Scale linearly within this bucket
                let left_point = if b == 0 { 0.0 } else { BUCKET_LIMIT[b - 1] };
                let right_point = BUCKET_LIMIT[b];
                let left_sum = sum - self.buckets[b];
                let right_sum = sum;
                let pos = (threshold - left_sum) / (right_sum - left_sum);
                // NOTE: Use if-else if-else tree?
                let mut r = left_point + (right_point - left_point) * pos;
                if r < self.min {
                    r = self.min;
                }
                if r > self.max {
                    r = self.max;
                }
                return r;
            }
        }
        self.max
    }

    fn average(&self) -> f64 {
        if self.num == 0.0 {
            0.0
        } else {
            self.sum / self.num
        }
    }

    fn standard_deviation(&self) -> f64 {
        if self.num == 0.0 {
            0.0
        } else {
            let variance =
                (self.sum_squares * self.num - self.sum * self.sum) / (self.num * self.num);
            variance.sqrt()
        }
    }
}

impl ToString for Histogram {
    fn to_string(&self) -> String {
        let mut r = format!(
            "Count: {:.0}  Average: {:.4}  StdDev: {:.2}\n",
            self.num,
            self.average(),
            self.standard_deviation(),
        );
        r += &format!(
            "Min: {:.4}  Median: {:.4}  Max: {:.2}\n",
            if self.num == 0.0 { 0.0 } else { self.min },
            self.median(),
            self.max,
        );
        r += "------------------------------------------------------\n";
        let mult = 100.0 / self.num;
        let mut sum = 0.0;
        for b in 0..NUM_BUCKETS {
            if self.buckets[b] <= 0.0 {
                continue;
            }
            sum += self.buckets[b];
            r += &format!(
                "[ {:7.0}, {:7.0} ) {:7.0} {:7.3}% {:7.3}% ",
                if b == 0 { 0.0 } else { BUCKET_LIMIT[b - 1] }, // left
                BUCKET_LIMIT[b],                                // right
                self.buckets[b],                                // count
                mult * self.buckets[b],                         // percentage
                mult * sum,                                     // cumulative percentage
            );
            // Add hash marks based on percentage; 20 marks for 100%.
            let marks = (20.0 * (self.buckets[b] / self.num) + 0.5) as usize;
            r += &"#".repeat(marks);
            r.push('\n');
        }
        r
    }
}

const NUM_BUCKETS: usize = 154;

const BUCKET_LIMIT: [f64; NUM_BUCKETS] = [
    1.0,
    2.0,
    3.0,
    4.0,
    5.0,
    6.0,
    7.0,
    8.0,
    9.0,
    10.0,
    12.0,
    14.0,
    16.0,
    18.0,
    20.0,
    25.0,
    30.0,
    35.0,
    40.0,
    45.0,
    50.0,
    60.0,
    70.0,
    80.0,
    90.0,
    100.0,
    120.0,
    140.0,
    160.0,
    180.0,
    200.0,
    250.0,
    300.0,
    350.0,
    400.0,
    450.0,
    500.0,
    600.0,
    700.0,
    800.0,
    900.0,
    1000.0,
    1200.0,
    1400.0,
    1600.0,
    1800.0,
    2000.0,
    2500.0,
    3000.0,
    3500.0,
    4000.0,
    4500.0,
    5000.0,
    6000.0,
    7000.0,
    8000.0,
    9000.0,
    10000.0,
    12000.0,
    14000.0,
    16000.0,
    18000.0,
    20000.0,
    25000.0,
    30000.0,
    35000.0,
    40000.0,
    45000.0,
    50000.0,
    60000.0,
    70000.0,
    80000.0,
    90000.0,
    100000.0,
    120000.0,
    140000.0,
    160000.0,
    180000.0,
    200000.0,
    250000.0,
    300000.0,
    350000.0,
    400000.0,
    450000.0,
    500000.0,
    600000.0,
    700000.0,
    800000.0,
    900000.0,
    1000000.0,
    1200000.0,
    1400000.0,
    1600000.0,
    1800000.0,
    2000000.0,
    2500000.0,
    3000000.0,
    3500000.0,
    4000000.0,
    4500000.0,
    5000000.0,
    6000000.0,
    7000000.0,
    8000000.0,
    9000000.0,
    10000000.0,
    12000000.0,
    14000000.0,
    16000000.0,
    18000000.0,
    20000000.0,
    25000000.0,
    30000000.0,
    35000000.0,
    40000000.0,
    45000000.0,
    50000000.0,
    60000000.0,
    70000000.0,
    80000000.0,
    90000000.0,
    100000000.0,
    120000000.0,
    140000000.0,
    160000000.0,
    180000000.0,
    200000000.0,
    250000000.0,
    300000000.0,
    350000000.0,
    400000000.0,
    450000000.0,
    500000000.0,
    600000000.0,
    700000000.0,
    800000000.0,
    900000000.0,
    1000000000.0,
    1200000000.0,
    1400000000.0,
    1600000000.0,
    1800000000.0,
    2000000000.0,
    2500000000.0,
    3000000000.0,
    3500000000.0,
    4000000000.0,
    4500000000.0,
    5000000000.0,
    6000000000.0,
    7000000000.0,
    8000000000.0,
    9000000000.0,
    1e200,
];
