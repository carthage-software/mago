<?php

declare(strict_types=1);

function redis_cluster_is_known(RedisCluster $cluster): bool
{
    $cluster->setOption(RedisCluster::OPT_SLAVE_FAILOVER, RedisCluster::FAILOVER_DISTRIBUTE);

    try {
        return $cluster->set('key', 'value') === true;
    } catch (RedisClusterException) {
        return false;
    }
}
