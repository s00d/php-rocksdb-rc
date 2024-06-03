FROM wordpress:php8.1-fpm
RUN echo 'rlimit_core = unlimited' >> /usr/local/etc/php-fpm.conf
RUN echo 'process.dumpable = yes' >> /usr/local/etc/php-fpm.conf
RUN apt-get update && apt-get install -y gdb vim
RUN pecl install igbinary redis && docker-php-ext-enable redis
RUN yes | pecl install xdebug && docker-php-ext-enable xdebug
# TO enable core dumps, you must run the following from a priviliaged container, unless you host is linux, in which case, run it directly.
# echo "/tmp/core.%e.%p.%h.%t" > /proc/sys/kernel/core_pattern
